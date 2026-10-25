use std::io::{Cursor, Read};
use std::path::Path;

use anyhow::{Context as _, Result, bail};
use bluepencil_core::{Document, Format};
use quick_xml::Reader;
use quick_xml::events::{BytesRef, BytesStart, BytesText, Event};

const MAX_XML_BYTES: usize = 64 * 1024 * 1024;
const MAX_ITEM_DEPTH: usize = 256;

fn extension(path: &Path) -> String {
    path.extension().and_then(|ext| ext.to_str()).unwrap_or_default().to_ascii_lowercase()
}

pub fn is_package_dir(path: &Path) -> bool {
    path.is_dir() && matches!(extension(path).as_str(), "scriv" | "ulysses" | "pages")
}

pub fn load(path: &Path) -> Result<Option<Vec<Document>>> {
    let name = crate::glob::display(path);
    let docs = match extension(path).as_str() {
        "fdx" => vec![Document::parse(name, fdx(&read_limited(path)?)?, Format::Fountain)],
        "tbx" => tbx(&name, &read_limited(path)?)?,
        "ulyz" => vec![Document::parse(name, ulyz(&read_limited(path)?)?, Format::Markdown)],
        "ulysses" if path.is_dir() => {
            let sheet = ["Content.xml", "Contents.xml"]
                .into_iter()
                .map(|file| path.join(file))
                .find(|file| file.is_file())
                .context("Ulysses package has no Content.xml sheet")?;
            let xml = read_limited(&sheet)?;
            vec![Document::parse(name, ulysses(&xml)?, Format::Markdown)]
        }
        "scriv" if path.is_dir() => scrivener(path, &name)?,
        "pages" => vec![pages(path, &name)?],
        _ => return Ok(None),
    };
    Ok(Some(docs))
}

fn read_limited(path: &Path) -> Result<Vec<u8>> {
    let file = std::fs::File::open(path).with_context(|| format!("opening {}", path.display()))?;
    let mut bytes = Vec::new();
    file.take((MAX_XML_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .with_context(|| format!("reading {}", path.display()))?;
    if bytes.len() > MAX_XML_BYTES {
        bail!("{} exceeds the 64 MiB import limit", path.display());
    }
    Ok(bytes)
}

fn attr(e: &BytesStart<'_>, key: &str) -> Result<Option<String>> {
    for attribute in e.attributes() {
        let attribute = attribute.context("reading XML attribute")?;
        if attribute.key.local_name().as_ref() == key {
            return Ok(Some(attribute.normalized_value(quick_xml::XmlVersion::Implicit1_0)?.into_owned()));
        }
    }
    Ok(None)
}

fn xml_text(t: &BytesText<'_>) -> Result<String> {
    Ok(quick_xml::escape::unescape(t.as_ref()).context("invalid XML text entity")?.into_owned())
}

fn xml_reference(reference: &BytesRef<'_>) -> Result<String> {
    if let Some(character) = reference.resolve_char_ref().context("invalid XML character reference")? {
        return Ok(character.to_string());
    }
    let value = match reference.as_ref() {
        "amp" => "&",
        "lt" => "<",
        "gt" => ">",
        "quot" => "\"",
        "apos" => "'",
        other => bail!("unsupported XML entity &{other};"),
    };
    Ok(value.to_string())
}

fn fdx(bytes: &[u8]) -> Result<String> {
    let xml = std::str::from_utf8(bytes).context("Final Draft file is not UTF-8 XML")?;
    let mut reader = Reader::from_str(xml);
    let mut in_content = false;
    let mut in_text = false;
    let mut paragraph: Option<(String, String)> = None;
    let mut out = String::new();
    loop {
        match reader.read_event().context("parsing Final Draft XML")? {
            Event::Eof => break,
            Event::Start(e) => match e.local_name().as_ref() {
                "Content" => in_content = true,
                "Paragraph" if in_content => paragraph = Some((attr(&e, "Type")?.unwrap_or_default(), String::new())),
                "Text" if paragraph.is_some() => in_text = true,
                _ => {}
            },
            Event::Text(t) if in_text => {
                if let Some((_, text)) = paragraph.as_mut() {
                    text.push_str(&xml_text(&t)?);
                }
            }
            Event::GeneralRef(reference) if in_text => {
                if let Some((_, text)) = paragraph.as_mut() {
                    text.push_str(&xml_reference(&reference)?);
                }
            }
            Event::CData(text) if in_text => {
                if let Some((_, body)) = paragraph.as_mut() {
                    body.push_str(text.as_ref());
                }
            }
            Event::End(e) => match e.local_name().as_ref() {
                "Text" => in_text = false,
                "Paragraph" => {
                    if let Some((kind, text)) = paragraph.take() {
                        let text = text.trim();
                        if !text.is_empty() {
                            match kind.as_str() {
                                "Character" => out.push_str(&format!("\n{}\n", text.to_uppercase())),
                                "Dialogue" | "Parenthetical" => out.push_str(&format!("{text}\n")),
                                _ => out.push_str(&format!("\n{text}\n\n")),
                            }
                        }
                    }
                }
                "Content" => in_content = false,
                _ => {}
            },
            _ => {}
        }
    }
    if out.trim().is_empty() {
        bail!("Final Draft file has no script paragraphs");
    }
    Ok(out)
}

#[derive(Default)]
struct Note {
    name: String,
    text: String,
    children: Vec<Note>,
}

#[derive(Clone, Copy)]
enum Field {
    Name,
    Text,
}

fn tbx(name: &str, bytes: &[u8]) -> Result<Vec<Document>> {
    let xml = std::str::from_utf8(bytes).context("Tinderbox file is not UTF-8 XML")?;
    let mut reader = Reader::from_str(xml);
    let mut stack = Vec::<Note>::new();
    let mut item_depths = Vec::<usize>::new();
    let mut roots = Vec::<Note>::new();
    let mut field = None;
    let mut depth = 0usize;
    loop {
        match reader.read_event().context("parsing Tinderbox XML")? {
            Event::Eof => break,
            Event::Start(e) => {
                depth += 1;
                match e.local_name().as_ref() {
                    "item" => {
                        if stack.len() >= MAX_ITEM_DEPTH {
                            bail!("Tinderbox note nesting exceeds {MAX_ITEM_DEPTH} levels");
                        }
                        stack.push(Note::default());
                        item_depths.push(depth);
                    }
                    "attribute"
                        if item_depths.last().is_some_and(|item_depth| depth == item_depth + 1)
                            && attr(&e, "name")?.as_deref() == Some("Name") =>
                    {
                        field = Some((Field::Name, depth));
                    }
                    "text" if item_depths.last().is_some_and(|item_depth| depth == item_depth + 1) => {
                        field = Some((Field::Text, depth));
                    }
                    _ => {}
                }
            }
            Event::Text(t) => {
                if let (Some(note), Some((field, field_depth))) = (stack.last_mut(), field)
                    && depth == field_depth
                {
                    match field {
                        Field::Name => note.name.push_str(&xml_text(&t)?),
                        Field::Text => note.text.push_str(&xml_text(&t)?),
                    }
                }
            }
            Event::GeneralRef(reference) => {
                if let (Some(note), Some((field, field_depth))) = (stack.last_mut(), field)
                    && depth == field_depth
                {
                    match field {
                        Field::Name => note.name.push_str(&xml_reference(&reference)?),
                        Field::Text => note.text.push_str(&xml_reference(&reference)?),
                    }
                }
            }
            Event::CData(text) => {
                if let (Some(note), Some((field, field_depth))) = (stack.last_mut(), field)
                    && depth == field_depth
                {
                    match field {
                        Field::Name => note.name.push_str(text.as_ref()),
                        Field::Text => note.text.push_str(text.as_ref()),
                    }
                }
            }
            Event::End(e) => {
                match e.local_name().as_ref() {
                    "attribute" | "text" if field.is_some_and(|(_, field_depth)| field_depth == depth) => {
                        field = None;
                    }
                    "item" => {
                        item_depths.pop();
                        if let Some(note) = stack.pop() {
                            if let Some(parent) = stack.last_mut() {
                                parent.children.push(note);
                            } else {
                                roots.push(note);
                            }
                        }
                    }
                    _ => {}
                }
                depth = depth.checked_sub(1).context("unbalanced Tinderbox XML")?;
            }
            _ => {}
        }
    }
    if depth != 0 || !stack.is_empty() {
        bail!("incomplete Tinderbox XML");
    }
    fn collect(note: Note, path: &str, docs: &mut Vec<Document>) {
        let title = if note.name.trim().is_empty() { "Untitled" } else { note.name.trim() };
        let path = format!("{path}/{title}");
        if !note.text.trim().is_empty() {
            let text = note.text.replace("\r\n", "\n").replace('\r', "\n");
            docs.push(Document::parse(path.clone(), text, Format::Plain));
        }
        for child in note.children {
            collect(child, &path, docs);
        }
    }
    let mut docs = Vec::new();
    for note in roots {
        collect(note, name, &mut docs);
    }
    if docs.is_empty() {
        bail!("Tinderbox document has no note text");
    }
    Ok(docs)
}

fn ulyz(bytes: &[u8]) -> Result<String> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).context("not a valid Ulysses .ulyz archive")?;
    for index in 0..archive.len() {
        let entry = archive.by_index(index).context("reading Ulysses archive")?;
        if ["Content.xml", "Contents.xml"]
            .iter()
            .any(|name| entry.name() == *name || entry.name().ends_with(&format!("/{name}")))
        {
            let mut xml = Vec::new();
            entry.take((MAX_XML_BYTES + 1) as u64).read_to_end(&mut xml).context("reading Ulysses sheet")?;
            if xml.len() > MAX_XML_BYTES {
                bail!("Ulysses sheet exceeds the 64 MiB import limit");
            }
            return ulysses(&xml);
        }
    }
    bail!("Ulysses archive has no Content.xml sheet");
}

fn ulysses(bytes: &[u8]) -> Result<String> {
    let xml = std::str::from_utf8(bytes).context("Ulysses sheet is not UTF-8 XML")?;
    let mut reader = Reader::from_str(xml);
    let mut in_string = false;
    let mut paragraph: Option<String> = None;
    let mut out = Vec::new();
    loop {
        match reader.read_event().context("parsing Ulysses XML")? {
            Event::Eof => break,
            Event::Start(e) => match e.local_name().as_ref() {
                "string" => in_string = true,
                "p" if in_string => paragraph = Some(String::new()),
                _ => {}
            },
            Event::Empty(e) if e.local_name().as_ref() == "tag" => {
                if let Some(p) = paragraph.as_mut()
                    && let Some(kind) = attr(&e, "kind")?
                    && let Some(level) = kind.strip_prefix("heading").and_then(|value| value.parse::<usize>().ok())
                    && (1..=6).contains(&level)
                {
                    p.push_str(&"#".repeat(level));
                    p.push(' ');
                }
            }
            Event::Text(t) => {
                if let Some(p) = paragraph.as_mut() {
                    p.push_str(&xml_text(&t)?);
                }
            }
            Event::GeneralRef(reference) => {
                if let Some(p) = paragraph.as_mut() {
                    p.push_str(&xml_reference(&reference)?);
                }
            }
            Event::CData(text) => {
                if let Some(p) = paragraph.as_mut() {
                    p.push_str(text.as_ref());
                }
            }
            Event::End(e) => match e.local_name().as_ref() {
                "p" => {
                    if let Some(p) = paragraph.take() {
                        out.push(p);
                    }
                }
                "string" => in_string = false,
                _ => {}
            },
            _ => {}
        }
    }
    if out.iter().all(|p| p.trim().is_empty()) {
        bail!("Ulysses sheet has no body text");
    }
    Ok(out.join("\n\n"))
}

fn pages(path: &Path, name: &str) -> Result<Document> {
    #[cfg(target_os = "macos")]
    {
        let dir = tempfile::tempdir().context("creating temporary Pages export directory")?;
        let output = dir.path().join("bluepencil.docx");
        let script = r#"on run argv
set inputFile to POSIX file (item 1 of argv)
set outputFile to POSIX file (item 2 of argv)
tell application "Pages"
    set wasOpen to false
    repeat with currentDoc in documents
        try
            if (file of currentDoc as alias) is (inputFile as alias) then set wasOpen to true
        end try
    end repeat
    set targetDoc to open inputFile
    export targetDoc to outputFile as Microsoft Word
    if not wasOpen then close targetDoc saving no
end tell
end run"#;
        let result = std::process::Command::new("osascript")
            .arg("-e")
            .arg(script)
            .arg(path)
            .arg(&output)
            .output()
            .context("launching Pages for native import")?;
        if !result.status.success() {
            bail!("Pages could not export {}: {}", path.display(), String::from_utf8_lossy(&result.stderr).trim());
        }
        let docx = read_limited(&output)?;
        let text = bluepencil_core::parse::docx::extract(&docx).context("reading Pages export")?;
        Ok(Document::parse(name, text, Format::Markdown))
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = name;
        bail!("native Pages import requires macOS with Pages installed: {}", path.display())
    }
}

#[derive(Default)]
struct BinderItem {
    kind: String,
    uuid: String,
    title: String,
    children: Vec<BinderItem>,
}

fn scrivener(path: &Path, name: &str) -> Result<Vec<Document>> {
    let manifest = std::fs::read_dir(path)
        .with_context(|| format!("reading Scrivener project {}", path.display()))?
        .collect::<std::io::Result<Vec<_>>>()
        .with_context(|| format!("reading Scrivener project {}", path.display()))?
        .into_iter()
        .map(|entry| entry.path())
        .find(|p| p.is_file() && extension(p) == "scrivx")
        .context("Scrivener project has no .scrivx binder")?;
    let bytes = read_limited(&manifest)?;
    let xml = std::str::from_utf8(&bytes).context("Scrivener binder is not UTF-8 XML")?;
    let mut reader = Reader::from_str(xml);
    let mut stack = Vec::<BinderItem>::new();
    let mut roots = Vec::<BinderItem>::new();
    let mut in_title = false;
    loop {
        match reader.read_event().context("parsing Scrivener binder")? {
            Event::Eof => break,
            Event::Start(e) => match e.local_name().as_ref() {
                "BinderItem" => {
                    if stack.len() >= MAX_ITEM_DEPTH {
                        bail!("Scrivener binder nesting exceeds {MAX_ITEM_DEPTH} levels");
                    }
                    stack.push(BinderItem {
                        kind: attr(&e, "Type")?.unwrap_or_default(),
                        uuid: attr(&e, "UUID")?.unwrap_or_default(),
                        ..Default::default()
                    });
                }
                "Title" if !stack.is_empty() => in_title = true,
                _ => {}
            },
            Event::Text(t) if in_title => {
                if let Some(item) = stack.last_mut() {
                    item.title.push_str(&xml_text(&t)?);
                }
            }
            Event::GeneralRef(reference) if in_title => {
                if let Some(item) = stack.last_mut() {
                    item.title.push_str(&xml_reference(&reference)?);
                }
            }
            Event::CData(text) if in_title => {
                if let Some(item) = stack.last_mut() {
                    item.title.push_str(text.as_ref());
                }
            }
            Event::End(e) => match e.local_name().as_ref() {
                "Title" => in_title = false,
                "BinderItem" => {
                    if let Some(item) = stack.pop() {
                        if let Some(parent) = stack.last_mut() {
                            parent.children.push(item);
                        } else {
                            roots.push(item);
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
    fn collect(item: BinderItem, project: &Path, prefix: &str, in_draft: bool, docs: &mut Vec<Document>) -> Result<()> {
        let draft = in_draft || item.kind == "DraftFolder";
        let title = if item.title.trim().is_empty() { "Untitled" } else { item.title.trim() };
        let title = title.replace(['/', '\\', ':'], " ");
        let label = format!("{prefix}/{title}");
        if draft && item.kind != "DraftFolder" && !item.uuid.is_empty() {
            if item.uuid.len() > 64 || !item.uuid.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-') {
                bail!("Scrivener binder contains an invalid document ID");
            }
            let rtf = project.join("Files/Data").join(&item.uuid).join("content.rtf");
            if rtf.is_file() {
                let bytes = read_limited(&rtf)?;
                let text = rtf_to_html::rtf_to_plain_text(&bytes)
                    .with_context(|| format!("extracting Scrivener text from {}", rtf.display()))?;
                if !text.trim().is_empty() {
                    docs.push(Document::parse(label.clone(), text, Format::Plain));
                }
            }
        }
        for child in item.children {
            collect(child, project, &label, draft, docs)?;
        }
        Ok(())
    }
    let mut docs = Vec::new();
    for root in roots {
        collect(root, path, name, false, &mut docs)?;
    }
    if docs.is_empty() {
        bail!("Scrivener project has no draft text in Files/Data");
    }
    Ok(docs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn final_draft_renders_dialogue_and_skips_metadata() {
        let xml = br#"<FinalDraft><Content><Paragraph Type="Scene Heading"><Text>INT. ROOM - DAY</Text></Paragraph><Paragraph Type="Character"><Text>MARA</Text></Paragraph><Paragraph Type="Dialogue"><Text>Hello &amp; goodbye.</Text></Paragraph></Content><Watermarking><Text>secret</Text></Watermarking></FinalDraft>"#;
        let text = fdx(xml).unwrap();
        assert!(text.contains("INT. ROOM - DAY"));
        assert!(text.contains("MARA\nHello & goodbye."), "{text:?}");
        assert!(!text.contains("secret"));
    }

    #[test]
    fn tinderbox_preserves_nested_note_names() {
        let xml = br#"<tinderbox><item><attribute name="Name">Chapter</attribute><item><attribute name="Name">Scene</attribute><text>She walked home.</text></item></item></tinderbox>"#;
        let docs = tbx("story.tbx", xml).unwrap();
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].name, "story.tbx/Chapter/Scene");
        assert_eq!(docs[0].word_count(), 3);
    }

    #[test]
    fn tinderbox_ignores_adornment_metadata_and_keeps_paragraphs() {
        let xml = br#"<tinderbox><item><attribute name="Name">Chapter</attribute><adornment><attribute name="Name">Map label</attribute><text>Decoration</text></adornment><item><attribute name="Name">Scene</attribute><text>First line.&#13;&#13;Second line.</text></item></item></tinderbox>"#;
        let docs = tbx("story.tbx", xml).unwrap();
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].name, "story.tbx/Chapter/Scene");
        assert_eq!(docs[0].source, "First line.\n\nSecond line.");
    }

    #[test]
    fn tinderbox_rejects_incomplete_document() {
        let xml = br#"<tinderbox><item><attribute name="Name">Scene</attribute><text>Words.</text></item>"#;
        assert!(tbx("story.tbx", xml).is_err());
    }

    #[test]
    fn tinderbox_nesting_limit() {
        for depth in [MAX_ITEM_DEPTH - 1, MAX_ITEM_DEPTH, MAX_ITEM_DEPTH + 1] {
            let xml = format!(
                "<tinderbox>{}<text>Words in a note.</text>{}</tinderbox>",
                "<item>".repeat(depth),
                "</item>".repeat(depth)
            );
            let result = tbx("notes.tbx", xml.as_bytes());
            assert_eq!(result.is_ok(), depth <= MAX_ITEM_DEPTH, "depth {depth}");
        }
    }

    #[test]
    fn ulysses_reads_body_and_heading_markup() {
        let xml = br##"<sheet><markup><tag definition="heading1">#</tag></markup><string><p><tags><tag kind="heading1"># </tag></tags>Chapter</p><p>She walked home.</p></string></sheet>"##;
        let text = ulysses(xml).unwrap();
        assert_eq!(text, "# Chapter\n\nShe walked home.");

        let with_empty_tag = br#"<sheet><string><p><tags><tag kind="heading2"/></tags>Scene</p></string></sheet>"#;
        assert_eq!(ulysses(with_empty_tag).unwrap(), "## Scene");
    }

    #[test]
    fn scrivener_reads_draft_in_binder_order_and_skips_research() {
        let dir = tempfile::tempdir().unwrap();
        let project = dir.path().join("story.scriv");
        std::fs::create_dir_all(project.join("Files/Data/ONE")).unwrap();
        std::fs::create_dir_all(project.join("Files/Data/TWO")).unwrap();
        std::fs::write(project.join("story.scrivx"), r#"<ScrivenerProject><Binder><BinderItem Type="DraftFolder" UUID="DRAFT"><Title>Draft</Title><Children><BinderItem Type="Text" UUID="ONE"><Title>Chapter One</Title></BinderItem></Children></BinderItem><BinderItem Type="ResearchFolder" UUID="RESEARCH"><Title>Research</Title><Children><BinderItem Type="Text" UUID="TWO"><Title>Notes</Title></BinderItem></Children></BinderItem></Binder></ScrivenerProject>"#).unwrap();
        std::fs::write(project.join("Files/Data/ONE/content.rtf"), br#"{\rtf1\ansi She walked home.}"#).unwrap();
        std::fs::write(project.join("Files/Data/TWO/content.rtf"), br#"{\rtf1\ansi Secret research.}"#).unwrap();
        let docs = scrivener(&project, "story.scriv").unwrap();
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].name, "story.scriv/Draft/Chapter One");
        assert_eq!(docs[0].word_count(), 3);
    }

    #[test]
    fn scrivener_nesting_limit() {
        let dir = tempfile::tempdir().unwrap();
        let project = dir.path().join("story.scriv");
        std::fs::create_dir_all(project.join("Files/Data/ONE")).unwrap();
        std::fs::write(project.join("Files/Data/ONE/content.rtf"), br#"{\rtf1\ansi She walked home.}"#).unwrap();
        for depth in [MAX_ITEM_DEPTH - 1, MAX_ITEM_DEPTH, MAX_ITEM_DEPTH + 1] {
            let xml = format!(
                "<ScrivenerProject><Binder>{}<BinderItem Type=\"Text\" UUID=\"ONE\"><Title>Chapter</Title></BinderItem>{}</Binder></ScrivenerProject>",
                "<BinderItem Type=\"DraftFolder\"><Title>Folder</Title><Children>".repeat(depth - 1),
                "</Children></BinderItem>".repeat(depth - 1)
            );
            std::fs::write(project.join("story.scrivx"), xml).unwrap();
            let result = scrivener(&project, "story.scriv");
            assert_eq!(result.is_ok(), depth <= MAX_ITEM_DEPTH, "depth {depth}");
        }
    }
}
