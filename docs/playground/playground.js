import init, { analyze } from "./pkg/bluepencil_web.js";

const source = document.getElementById("source");
const format = document.getElementById("format");
const findings = document.getElementById("findings");
const filterBar = document.getElementById("filter-bar");
const status = document.getElementById("status");
const stats = document.getElementById("stats");
const button = document.getElementById("analyze");
const annotated = document.getElementById("annotated");
const dropZone = document.getElementById("drop-zone");
const fileInput = document.getElementById("file-input");
const workspace = document.getElementById("workspace");
const mobileTabs = [document.getElementById("mobile-editor"), document.getElementById("mobile-results")];
const mobileCount = document.getElementById("mobile-count");
const compactLayout = window.matchMedia("(max-width: 900px)");
const tabs = [document.getElementById("findings-tab"), document.getElementById("annotated-tab")];
const panels = [document.getElementById("findings-view"), document.getElementById("annotated-view")];

const samples = {
  fiction: {
    format: "markdown",
    text: `# The Letter

Mara crossed the hotel lobby with the envelope against her ribs. The lobby smelled of wet wool and coffee. At the far desk, Ames folded a map, then unfolded it.

“You came back,” he said.

“I came for the letter.”

She looked at the staircase. She looked at the empty chair beside it. She looked at the door that had not quite latched.

The envelope was sealed with blue wax. She knew the name on it, though she had never seen the handwriting. When Ames reached for it, she pulled it away.

“I just need a minute,” she said quietly.

Outside, rain tapped the glass. The glass shivered in its frame. She waited for him to ask the question he had avoided all morning.`,
  },
  screenplay: {
    format: "fountain",
    text: `INT. TRAIN STATION - NIGHT

The departure board flickers. The departure board goes dark. MARA stands beneath it, holding a small blue suitcase.

AMES
You missed the last train.

MARA
I know.

She looks toward the empty platform. She looks at the suitcase. She looks at him.

AMES
You could stay.

MARA
I could.

The station clock was stopped at eleven. Rain ticked against the glass. Mara carefully set the suitcase down, then picked it up again.

AMES
What are you waiting for?

Mara turns toward the platform. A train whistle sounds somewhere beyond the dark.`,
  },
};

const priority = { cliche: 90, "repeated-starter": 85, monotony: 80, "long-sentence": 75, passive: 70, filter: 60, hedge: 55, echo: 50, adverb: 40 };
let ready = false;
let current = [];
let focus = [];
let selectedRule = "focus";
let debounce;

function setMobilePanel(panel) {
  workspace.dataset.mobilePanel = panel;
  mobileTabs.forEach((tab, index) => {
    const selected = (index === 0 ? "editor" : "results") === panel;
    tab.setAttribute("aria-selected", String(selected));
    tab.tabIndex = selected ? 0 : -1;
  });
}

function setTab(index) {
  tabs.forEach((tab, at) => {
    tab.setAttribute("aria-selected", String(at === index));
    tab.tabIndex = at === index ? 0 : -1;
    panels[at].hidden = at !== index;
  });
}

function selectText(item) {
  if (compactLayout.matches) setMobilePanel("editor");
  source.focus();
  source.setSelectionRange(item.start, item.end);
}

function visibleIndices() {
  if (selectedRule === "focus") return focus;
  return current.map((item, index) => index).filter((index) => selectedRule === "all" || current[index].rule === selectedRule);
}

function showFinding(item, index) {
  const li = document.createElement("li");
  const action = document.createElement("button");
  action.type = "button";
  action.dataset.index = String(index);
  action.setAttribute("aria-label", `${item.rule}, line ${item.line}, column ${item.column}: ${item.message}. Jump to text.`);
  const top = document.createElement("span");
  top.className = "finding-top";
  const location = document.createElement("span");
  location.className = "location";
  location.textContent = `${String(item.line).padStart(2, "0")}:${String(item.column).padStart(2, "0")}`;
  const rule = document.createElement("span");
  rule.className = "rule";
  rule.textContent = item.rule.replaceAll("-", " ");
  const message = document.createElement("span");
  message.className = "message";
  message.textContent = item.message;
  const excerpt = document.createElement("span");
  excerpt.className = "excerpt";
  excerpt.textContent = item.excerpt;
  top.append(location, rule);
  action.append(top, message, excerpt);
  action.addEventListener("click", () => selectText(item));
  li.append(action);
  findings.append(li);
}

function renderFindings() {
  findings.replaceChildren();
  for (const index of visibleIndices()) showFinding(current[index], index);
  const patterns = new Set(current.map((item) => item.rule)).size;
  status.textContent = current.length
    ? compactLayout.matches
      ? `${current.length} observations · ${patterns} patterns`
      : `${current.length} observation${current.length === 1 ? "" : "s"} across ${patterns} patterns. Start with the focused set or explore every line.`
    : source.value.trim() ? "Nothing stands out in this passage. Try a longer sample." : "Write or open a passage to begin.";
}

function renderFilters() {
  filterBar.replaceChildren();
  const rules = [...new Set(current.map((item) => item.rule))];
  if (selectedRule !== "focus" && selectedRule !== "all" && !rules.includes(selectedRule)) selectedRule = "focus";
  filterBar.hidden = !current.length;
  for (const value of ["focus", "all", ...rules]) {
    const control = document.createElement("button");
    control.type = "button";
    const count = value === "focus" ? focus.length : value === "all" ? current.length : current.filter((item) => item.rule === value).length;
    control.textContent = `${value === "focus" ? "Start here" : value.replaceAll("-", " ")} ${count}`;
    control.setAttribute("aria-pressed", String(selectedRule === value));
    control.addEventListener("click", () => {
      selectedRule = value;
      renderFilters();
      renderFindings();
    });
    filterBar.append(control);
  }
}

function renderAnnotated() {
  annotated.replaceChildren();
  const text = source.value;
  const valid = current.map((item, index) => ({ ...item, index })).filter((item) => Number.isInteger(item.start) && Number.isInteger(item.end) && item.start >= 0 && item.start < item.end && item.end <= text.length);
  const offsets = [...new Set([0, text.length, ...valid.flatMap((item) => [item.start, item.end])])].sort((a, b) => a - b);
  for (let at = 0; at < offsets.length - 1; at++) {
    const start = offsets[at];
    const end = offsets[at + 1];
    const segment = text.slice(start, end);
    const match = valid.filter((item) => item.start <= start && item.end >= end).sort((a, b) => (priority[b.rule] || 0) - (priority[a.rule] || 0))[0];
    if (!match) {
      annotated.append(document.createTextNode(segment));
      continue;
    }
    const mark = document.createElement("button");
    mark.type = "button";
    mark.className = "marked-token";
    mark.textContent = segment;
    mark.setAttribute("aria-label", `${segment.trim() || "Marked passage"}: ${match.message}`);
    mark.title = match.message;
    mark.addEventListener("click", () => {
      selectedRule = "all";
      setTab(0);
      renderFilters();
      renderFindings();
      const target = [...findings.querySelectorAll("button[data-index]")].find((item) => item.dataset.index === String(match.index));
      target?.scrollIntoView({ block: "nearest" });
      target?.focus();
    });
    annotated.append(mark);
  }
}

function chooseFocus() {
  const ranked = current.map((item, index) => index).sort((a, b) => (priority[current[b].rule] || 0) - (priority[current[a].rule] || 0) || current[a].start - current[b].start);
  const seen = new Set();
  const picks = [];
  for (const index of ranked) {
    if (seen.has(current[index].rule)) continue;
    seen.add(current[index].rule);
    picks.push(index);
    if (picks.length === 6) break;
  }
  focus = picks;
}

function run() {
  if (!ready) return;
  try {
    const result = JSON.parse(analyze(source.value, format.value));
    current = result.findings;
    chooseFocus();
    stats.textContent = `${result.words} words · ${result.sentences} sentences`;
    mobileCount.textContent = String(current.length);
    mobileCount.hidden = current.length === 0;
    renderFilters();
    renderFindings();
    renderAnnotated();
    if (result.truncated) status.textContent += " Showing the first 150; use the CLI for a full report.";
  } catch (error) {
    current = [];
    focus = [];
    findings.replaceChildren();
    annotated.replaceChildren();
    filterBar.hidden = true;
    stats.textContent = "";
    mobileCount.hidden = true;
    status.textContent = String(error);
  }
}

function clearSampleSelection() {
  for (const control of document.querySelectorAll(".sample-choice")) control.setAttribute("aria-pressed", "false");
}

async function openFile(file) {
  if (!file) return;
  if (!/\.(md|markdown|mdx|txt|fountain|spmd)$/i.test(file.name)) {
    status.textContent = "Open a Markdown, Fountain, or plain text file here. Use the CLI for other writing app formats.";
    return;
  }
  if (file.size > 100_000) {
    status.textContent = "This file exceeds the 100 KB playground limit. Use the CLI for longer manuscripts.";
    return;
  }
  try {
    const text = new TextDecoder("utf-8", { fatal: true }).decode(await file.arrayBuffer());
    if (text.includes("\0")) throw new Error("This file does not appear to be plain text.");
    source.value = text;
    format.value = /\.(fountain|spmd)$/i.test(file.name) ? "fountain" : /\.txt$/i.test(file.name) ? "plain" : "markdown";
    selectedRule = "focus";
    clearSampleSelection();
    if (compactLayout.matches) setMobilePanel("editor");
    run();
  } catch (error) {
    status.textContent = error instanceof TypeError ? "This file is not UTF-8 text." : String(error);
  }
}

source.addEventListener("input", () => {
  clearSampleSelection();
  clearTimeout(debounce);
  debounce = setTimeout(run, 350);
});
compactLayout.addEventListener("change", renderFindings);
format.addEventListener("change", run);
button.addEventListener("click", () => {
  clearTimeout(debounce);
  run();
  if (compactLayout.matches) setMobilePanel("results");
});
for (const control of document.querySelectorAll(".sample-choice")) {
  control.addEventListener("click", () => {
    const sample = samples[control.dataset.sample];
    source.value = sample.text;
    format.value = sample.format;
    selectedRule = "focus";
    clearSampleSelection();
    control.setAttribute("aria-pressed", "true");
    if (compactLayout.matches) setMobilePanel("editor");
    run();
  });
}
document.getElementById("clear").addEventListener("click", () => {
  source.value = "";
  selectedRule = "focus";
  clearSampleSelection();
  if (compactLayout.matches) setMobilePanel("editor");
  run();
  source.focus();
});
document.getElementById("open-file").addEventListener("click", () => fileInput.click());
fileInput.addEventListener("change", () => {
  openFile(fileInput.files?.[0]);
  fileInput.value = "";
});
dropZone.addEventListener("dragover", (event) => {
  if (!event.dataTransfer?.types.includes("Files")) return;
  event.preventDefault();
  dropZone.classList.add("dragging");
});
dropZone.addEventListener("dragleave", () => dropZone.classList.remove("dragging"));
dropZone.addEventListener("drop", (event) => {
  event.preventDefault();
  dropZone.classList.remove("dragging");
  if (event.dataTransfer.files.length !== 1) {
    status.textContent = "Open one text file at a time.";
    return;
  }
  openFile(event.dataTransfer.files[0]);
});
tabs.forEach((tab, index) => {
  tab.addEventListener("click", () => setTab(index));
  tab.addEventListener("keydown", (event) => {
    if (event.key !== "ArrowLeft" && event.key !== "ArrowRight") return;
    event.preventDefault();
    const next = (index + (event.key === "ArrowRight" ? 1 : tabs.length - 1)) % tabs.length;
    setTab(next);
    tabs[next].focus();
  });
});
mobileTabs.forEach((tab, index) => {
  tab.addEventListener("click", () => setMobilePanel(index === 0 ? "editor" : "results"));
  tab.addEventListener("keydown", (event) => {
    if (event.key !== "ArrowLeft" && event.key !== "ArrowRight") return;
    event.preventDefault();
    const next = (index + (event.key === "ArrowRight" ? 1 : mobileTabs.length - 1)) % mobileTabs.length;
    setMobilePanel(next === 0 ? "editor" : "results");
    mobileTabs[next].focus();
  });
});
document.getElementById("copy-findings").addEventListener("click", async () => {
  if (!current.length) {
    status.textContent = "There are no findings to copy yet.";
    return;
  }
  const text = current.map((item) => `${item.line}:${item.column}  ${item.rule}  ${item.message}\n    ${item.excerpt}`).join("\n");
  try {
    await navigator.clipboard.writeText(text);
    status.textContent = `Copied ${current.length} findings to the clipboard.`;
  } catch {
    status.textContent = "Clipboard access was unavailable. Select and copy individual findings instead.";
  }
});

source.value = samples.fiction.text;
button.disabled = true;
try {
  await init();
  ready = true;
  button.disabled = false;
  run();
} catch {
  status.textContent = "The local analysis engine could not load. Reload this page to try again.";
}
