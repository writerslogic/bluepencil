use std::io::{IsTerminal, Read};

use anyhow::{Context as _, Result, bail};
use bluepencil_core::lexicon::{Lexicons, WordSet, user};
use bluepencil_core::{Document, Format};

use crate::cli::{Global, Input, InputFormat};
use crate::config::{self, Config};
use crate::glob;

pub struct Context {
    pub config: Config,
    pub lexicons: Lexicons,
    pub json: bool,
    pub limit: usize,
    base: std::path::PathBuf,
    format: Option<InputFormat>,
}

fn to_format(f: InputFormat) -> Option<Format> {
    match f {
        InputFormat::Plain => Some(Format::Plain),
        InputFormat::Markdown => Some(Format::Markdown),
        InputFormat::Fountain => Some(Format::Fountain),
        InputFormat::Docx => None,
    }
}

fn is_docx_path(path: &std::path::Path) -> bool {
    path.extension().and_then(|e| e.to_str()).is_some_and(|e| e.eq_ignore_ascii_case("docx"))
}

impl Context {
    pub fn new(global: &Global) -> Result<Self> {
        let (config, base) = config::load(global.config.as_deref())?;
        let lexicons = build_lexicons(&config, &base)?;
        Ok(Self { config, lexicons, json: global.json, limit: global.limit, base, format: global.input_format })
    }

    pub fn documents(&self, input: &Input) -> Result<Vec<Document>> {
        let cwd = std::env::current_dir()?;
        let (patterns, base) = if !input.files.is_empty() {
            (input.files.clone(), cwd)
        } else if !self.config.project.files.is_empty() {
            (self.config.project.files.clone(), self.base.clone())
        } else if !std::io::stdin().is_terminal() {
            (vec!["-".to_string()], cwd)
        } else {
            bail!("no input files (pass paths, pipe text on stdin, or set project.files in bluepencil.toml)");
        };

        let mut docs = Vec::new();
        let (stdin, paths): (Vec<_>, Vec<_>) = patterns.into_iter().partition(|p| p == "-");
        if !stdin.is_empty() {
            if matches!(self.format, Some(InputFormat::Docx)) {
                bail!("`--as docx` is not supported on stdin; docx is a binary archive, pass a file path instead");
            }
            let mut text = String::new();
            std::io::stdin().read_to_string(&mut text).context("reading stdin")?;
            let format = self.format.and_then(to_format).unwrap_or(Format::Markdown);
            docs.push(Document::parse("<stdin>", text, format));
        }
        for path in glob::expand(&paths, &base)? {
            let bytes = std::fs::read(&path).with_context(|| format!("reading {}", path.display()))?;
            let as_docx =
                matches!(self.format, Some(InputFormat::Docx)) || (self.format.is_none() && is_docx_path(&path));
            if as_docx {
                let text = bluepencil_core::parse::docx::extract(&bytes)
                    .with_context(|| format!("reading {} as .docx", path.display()))?;
                docs.push(Document::parse(glob::display(&path), text, Format::Markdown));
                continue;
            }
            let text = String::from_utf8(bytes).with_context(|| format!("{} is not UTF-8 text", path.display()))?;
            let format = self.format.and_then(to_format).unwrap_or_else(|| Format::from_path(&path));
            docs.push(Document::parse(glob::display(&path), text, format));
        }
        Ok(docs)
    }

    pub fn echo_ignore(&self) -> WordSet {
        WordSet::from_list(self.config.echoes.ignore.iter().map(String::as_str))
    }

    pub fn cache(&self) -> crate::cache::Cache {
        crate::cache::Cache::open(&self.base, &self.config)
    }
}

fn strs(v: &[String]) -> Vec<&str> {
    v.iter().map(String::as_str).collect()
}

fn build_lexicons(config: &Config, base: &std::path::Path) -> Result<Lexicons> {
    let mut lex = Lexicons::default();
    let l = &config.lexicon;
    lex.stopwords.extend(strs(&l.stopwords));
    lex.not_adverbs.extend(strs(&l.not_adverbs));
    lex.filter.extend(strs(&l.filter));
    lex.hedges.extend(strs(&l.hedges));
    lex.cliches.extend(strs(&l.cliches));
    lex.tics.extend(strs(&config.tics.words));

    for (name, paths) in &l.files {
        for path in paths {
            let words = user::read_list(&base.join(path)).with_context(|| format!("reading {}", path.display()))?;
            let words = strs(&words);
            match name.as_str() {
                "stopwords" => lex.stopwords.extend(words),
                "not_adverbs" => lex.not_adverbs.extend(words),
                "filter" => lex.filter.extend(words),
                "hedges" => lex.hedges.extend(words),
                "cliches" => lex.cliches.extend(words),
                "tics" => lex.tics.extend(words),
                other => bail!("unknown list `{other}` in lexicon.files"),
            }
        }
    }

    let ignore = strs(&l.ignore);
    lex.filter.remove_all(ignore.iter().copied());
    lex.hedges.remove_all(ignore.iter().copied());
    lex.cliches.remove_all(ignore.iter().copied());
    lex.tics.remove_all(ignore.iter().copied());
    lex.not_adverbs.extend(ignore.iter().copied());
    Ok(lex)
}
