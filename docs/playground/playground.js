import { createEditor } from "./editor.js";
import { createChart } from "./chart.js";
import { MAX_INPUT_BYTES, groupFor, groups, rules } from "./review.js";

const source = document.getElementById("source");
const format = document.getElementById("format");
const button = document.getElementById("analyze");
const stats = document.getElementById("stats");
const status = document.getElementById("viewport-summary");
const notice = document.getElementById("analysis-notice");
const noticeMessage = document.getElementById("analysis-message");
const progress = document.getElementById("analysis-progress");
const cancelButton = document.getElementById("cancel-analysis");
const fileInput = document.getElementById("file-input");
const editorWrap = document.getElementById("editor-wrap");
const dropHint = document.getElementById("drop-hint");
const findingPanel = document.getElementById("finding-panel");
const chartPanel = document.getElementById("chart-panel");
const chartToggle = document.getElementById("toggle-chart");
const number = new Intl.NumberFormat("en").format;
const WORKER_URL = typeof BLUEPENCIL_WORKER_URL === "string" ? BLUEPENCIL_WORKER_URL : "./analysis-worker.js";

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

let worker;
let revision = 0;
let query = 0;
let running = false;
let summary;
let viewport;
let visible = [];
let visibleCounts = {};
let visibleResult;
let detailItems = [];
let detailIndex = 0;
let replacing = false;
let dirty = false;
let fileRead = 0;
let filename = "manuscript.md";
let debounce;
let frame;
let lastViewportKey = "";
let lastChartKey = "";
let lastDrawer;
const enabled = new Set(Object.keys(rules));
const channels = { visible: { pending: false }, chart: { pending: false } };
const toggleElements = [];

const chart = createChart({
  request: () => { lastChartKey = ""; requestChart(); },
  jump: (position) => editor.jump(position),
});
const editor = createEditor(source, samples.fiction.text, {
  change: documentChanged,
  viewport: () => scheduleViewport(),
  inspect: (position) => inspect(position),
  close: () => closeFinding(),
});

function setDrawer(panel, open) {
  if (open) {
    lastDrawer = panel;
    if (matchMedia("(max-width: 780px), (max-height: 650px)").matches) {
      const other = panel === chartPanel ? findingPanel : chartPanel;
      if (other.dataset.open === "true") setDrawer(other, false);
    }
  }
  panel.dataset.open = String(open);
  panel.setAttribute("aria-hidden", String(!open));
  panel.inert = !open;
  if (panel === chartPanel) {
    chartToggle.setAttribute("aria-expanded", String(open));
    chartToggle.textContent = open ? "Hide chart ⌄" : "Analysis chart ⌃";
  }
  scheduleViewport();
}

function closeFinding() {
  const open = findingPanel.dataset.open === "true";
  setDrawer(findingPanel, false);
  return open;
}

function showDetails(items, index = 0) {
  if (!items.length) return false;
  detailItems = items;
  detailIndex = (index + items.length) % items.length;
  const item = items[detailIndex];
  const detail = rules[item.rule];
  const group = groupFor(item.rule);
  const category = document.getElementById("finding-category");
  category.textContent = `${detail?.label || item.rule} · line ${number(item.line)}, column ${item.column}`;
  category.style.color = group?.color || "";
  document.getElementById("finding-heading").textContent = item.message;
  document.getElementById("finding-excerpt").textContent = item.excerpt;
  document.getElementById("finding-explanation").textContent = detail?.why || "An observation to consider in context.";
  const overlap = document.getElementById("finding-overlap");
  overlap.replaceChildren();
  if (items.length > 1) {
    items.forEach((finding, at) => {
      const control = document.createElement("button");
      control.type = "button";
      control.textContent = rules[finding.rule]?.label || finding.rule;
      control.setAttribute("aria-pressed", String(at === detailIndex));
      control.addEventListener("click", () => showDetails(items, at));
      overlap.append(control);
    });
  }
  document.getElementById("previous-finding").disabled = visible.length < 2;
  document.getElementById("next-finding").disabled = visible.length < 2;
  setDrawer(findingPanel, true);
  return true;
}

function inspect(position, nearest = false) {
  const matches = visible.filter((item) => item.start <= position && position <= item.end).sort((a, b) => b.priority - a.priority);
  if (matches.length) return showDetails(matches);
  if (!nearest || !visible.length) return false;
  const item = visible.reduce((best, candidate) => Math.abs(candidate.start - position) < Math.abs(best.start - position) ? candidate : best);
  editor.jump(item.start, item.end);
  return showDetails([item]);
}

function stepFinding(direction) {
  if (!visible.length) return;
  const current = detailItems[detailIndex];
  let index = visible.findIndex((item) => current && item.rule === current.rule && item.start === current.start && item.end === current.end);
  index = (index + direction + visible.length) % visible.length;
  const item = visible[index];
  editor.jump(item.start, item.end);
  showDetails([item]);
}

function buildToggles() {
  const container = document.getElementById("finding-toggles");
  for (const group of groups) {
    const details = document.createElement("details");
    details.className = "finding-group";
    details.dataset.group = group.id;
    details.style.setProperty("--finding-color", group.color);
    const title = document.createElement("summary");
    const swatch = document.createElement("span");
    swatch.className = "swatch";
    swatch.setAttribute("aria-hidden", "true");
    const count = document.createElement("span");
    count.className = "group-count";
    title.append(swatch, document.createTextNode(group.label), count);
    const popover = document.createElement("div");
    popover.className = "rule-popover";
    const action = document.createElement("button");
    action.type = "button";
    action.className = "text-button group-action";
    action.addEventListener("click", () => {
      const hide = group.rules.some((rule) => enabled.has(rule));
      for (const rule of group.rules) { if (hide) enabled.delete(rule); else enabled.add(rule); }
      filtersChanged();
    });
    popover.append(action);
    for (const rule of group.rules) {
      const label = document.createElement("label");
      label.className = "rule-option";
      const checkbox = document.createElement("input");
      checkbox.type = "checkbox";
      checkbox.checked = true;
      checkbox.dataset.rule = rule;
      checkbox.addEventListener("change", () => {
        if (checkbox.checked) enabled.add(rule); else enabled.delete(rule);
        filtersChanged();
      });
      const count = document.createElement("span");
      count.className = "rule-count";
      count.dataset.ruleCount = rule;
      label.append(checkbox, document.createTextNode(rules[rule].plural), count);
      popover.append(label);
    }
    details.append(title, popover);
    details.addEventListener("toggle", () => {
      if (!details.open) return;
      for (const other of toggleElements) if (other.details !== details) other.details.open = false;
      const rect = title.getBoundingClientRect();
      popover.style.setProperty("--popover-top", `${rect.bottom + 7}px`);
      popover.style.setProperty("--popover-left", `${Math.max(12, Math.min(rect.left, innerWidth - 282))}px`);
    });
    toggleElements.push({ details, count, action, group });
    container.append(details);
  }
  updateToggleCounts();
}

function updateToggleCounts() {
  for (const { details, count, action, group } of toggleElements) {
    const any = group.rules.some((rule) => enabled.has(rule));
    details.dataset.enabled = String(any);
    count.textContent = String(group.rules.reduce((sum, rule) => sum + (enabled.has(rule) ? visibleCounts[rule] || 0 : 0), 0));
    action.textContent = any ? `Hide all ${group.label.toLowerCase()} findings` : `Show all ${group.label.toLowerCase()} findings`;
  }
  for (const checkbox of document.querySelectorAll("input[data-rule]")) checkbox.checked = enabled.has(checkbox.dataset.rule);
  for (const count of document.querySelectorAll("[data-rule-count]")) count.textContent = number(visibleCounts[count.dataset.ruleCount] || 0);
}

function filtersChanged() {
  visible = visible.filter((item) => enabled.has(item.rule));
  editor.highlight(visible);
  updateToggleCounts();
  lastViewportKey = "";
  scheduleViewport();
}

function queueRequest(type, payload) {
  if (!worker || !summary) return;
  const channel = channels[type];
  const message = { type, id: revision, query: ++query, ...payload };
  channel.latest = message.query;
  if (channel.pending) channel.queued = message;
  else {
    channel.pending = true;
    worker.postMessage(message);
  }
}

function finishRequest(type, data, apply) {
  const channel = channels[type];
  channel.pending = false;
  if (data.query === channel.latest) apply();
  if (channel.queued && worker) {
    const queued = channel.queued;
    channel.queued = undefined;
    channel.pending = true;
    worker.postMessage(queued);
  }
}

function scheduleViewport() {
  if (frame) return;
  frame = requestAnimationFrame(() => {
    frame = undefined;
    viewport = editor.range();
    chart.viewport(viewport);
    if (!summary || !worker) return;
    const key = `${revision}:${viewport.from}:${viewport.to}:${[...enabled].join(",")}`;
    if (key !== lastViewportKey) {
      lastViewportKey = key;
      queueRequest("visible", { from: viewport.from, to: viewport.to, enabled: [...enabled] });
    } else if (visibleResult) renderViewportStatus();
    requestChart();
  });
}

function requestChart() {
  if (!summary || !worker || chartPanel.dataset.open !== "true") return;
  if (!viewport) { scheduleViewport(); return; }
  const from = chart.scope() === "whole" ? 0 : viewport.from;
  const to = chart.scope() === "whole" ? summary.length : viewport.to;
  const interval = chart.interval();
  const key = `${revision}:${chart.scope()}:${from}:${to}:${interval}`;
  if (key === lastChartKey) return;
  lastChartKey = key;
  queueRequest("chart", { from, to, interval });
}

function applyVisible(result) {
  visibleResult = result;
  visible = result.findings;
  visibleCounts = result.rule_counts;
  editor.highlight(visible);
  updateToggleCounts();
  renderViewportStatus();
}

function renderViewportStatus() {
  const result = visibleResult;
  const lines = `lines ${number(viewport.fromLine)}–${number(viewport.toLine)} + buffer`;
  status.textContent = result.total
    ? `${number(result.total)} observations near this passage · ${lines}${result.truncated ? " · showing the first 1,000" : ""}`
    : `${enabled.size ? "No observations" : "Highlights hidden"} near this passage · ${lines}`;
  status.title = status.textContent;
}

function stopWorker() {
  worker?.terminate();
  worker = undefined;
  running = false;
  button.disabled = false;
  cancelButton.hidden = true;
  for (const channel of Object.values(channels)) { channel.pending = false; channel.queued = undefined; }
}

function clearAnalysis(message, clearHighlights = true) {
  summary = undefined;
  visible = [];
  visibleCounts = {};
  visibleResult = undefined;
  lastViewportKey = "";
  lastChartKey = "";
  stats.textContent = "";
  status.textContent = message;
  if (clearHighlights) editor.highlight([]);
  chart.clear();
  updateToggleCounts();
  closeFinding();
}

function showError(message) {
  notice.dataset.kind = "error";
  notice.hidden = false;
  noticeMessage.textContent = message;
  progress.hidden = true;
  cancelButton.hidden = !running;
}

function run() {
  clearTimeout(debounce);
  stopWorker();
  const id = ++revision;
  clearAnalysis("Analyzing your manuscript…");
  notice.hidden = true;
  const text = editor.text();
  if (!text.trim()) { status.textContent = "Write, paste, or open your manuscript to begin."; return; }
  if (new TextEncoder().encode(text).length > MAX_INPUT_BYTES) {
    showError("This manuscript exceeds the 16 MiB browser limit. Open a smaller document or use the CLI.");
    return;
  }
  try {
    worker = new Worker(new URL(WORKER_URL, import.meta.url), { type: "module" });
    const active = worker;
    running = true;
    button.disabled = true;
    notice.dataset.kind = "progress";
    notice.hidden = false;
    noticeMessage.textContent = "Preparing your manuscript…";
    progress.hidden = false;
    progress.value = 0;
    cancelButton.hidden = false;
    worker.onmessage = ({ data }) => {
      if (worker !== active || data.id !== revision) return;
      if (data.type === "progress") {
        if (notice.dataset.kind === "error") return;
        noticeMessage.textContent = data.message;
        progress.max = data.total;
        progress.value = data.completed;
      } else if (data.type === "complete") {
        running = false;
        button.disabled = false;
        cancelButton.hidden = true;
        if (notice.dataset.kind === "progress") notice.hidden = true;
        summary = data.summary;
        stats.textContent = `${number(summary.words)} words · ${number(summary.total_findings)} observations`;
        scheduleViewport();
      } else if (data.type === "visible") {
        finishRequest("visible", data, () => applyVisible(data.result));
      } else if (data.type === "chart") {
        finishRequest("chart", data, () => chart.update(data.result));
      } else if (data.type === "error") {
        if (data.request) finishRequest(data.request, data, () => showError(data.message));
        else {
          stopWorker();
          clearAnalysis("Your text is still in the editor.");
          showError(data.message);
        }
      }
    };
    worker.onerror = (event) => {
      if (worker !== active) return;
      event.preventDefault();
      stopWorker();
      clearAnalysis("Your text is still in the editor.");
      showError("The local analysis engine could not finish. Try again or reload the page.");
    };
    worker.postMessage({ type: "analyze", id, text, format: format.value });
  } catch {
    stopWorker();
    showError("The local analysis engine could not start. Try reloading this page in a current browser.");
  }
}

function clearSampleSelection() {
  for (const control of document.querySelectorAll(".sample-choice")) control.setAttribute("aria-pressed", "false");
}

function documentChanged() {
  dirty = true;
  if (replacing) return;
  clearSampleSelection();
  clearTimeout(debounce);
  stopWorker();
  revision++;
  notice.hidden = true;
  clearAnalysis("Waiting for you to finish typing…", false);
  debounce = setTimeout(run, editor.length() > 100_000 ? 1000 : 350);
}

function replaceDocument(text, nextFormat) {
  replacing = true;
  editor.replace(text);
  replacing = false;
  format.value = nextFormat;
  run();
  editor.jump(0);
}

async function openFile(file) {
  if (!file) return;
  const reading = ++fileRead;
  if (!/\.(md|markdown|mdx|txt|fountain|spmd)$/i.test(file.name)) {
    showError("Open a Markdown, Fountain, or plain text file here. Use the CLI for other writing app formats.");
    return;
  }
  if (file.size > MAX_INPUT_BYTES) {
    showError("This file exceeds the 16 MiB browser limit. Open a smaller document or use the CLI.");
    return;
  }
  clearTimeout(debounce);
  const originalRevision = revision;
  try {
    const text = new TextDecoder("utf-8", { fatal: true }).decode(await file.arrayBuffer());
    if (reading !== fileRead || originalRevision !== revision) return;
    if (text.includes("\0")) throw new Error("This file does not appear to be plain text.");
    filename = file.name;
    clearSampleSelection();
    replaceDocument(text, /\.(fountain|spmd)$/i.test(file.name) ? "fountain" : /\.txt$/i.test(file.name) ? "plain" : "markdown");
  } catch (error) {
    if (reading !== fileRead || originalRevision !== revision) return;
    showError(error instanceof TypeError ? "This file is not UTF-8 text." : String(error));
  }
}

buildToggles();
button.addEventListener("click", run);
format.addEventListener("change", run);
cancelButton.addEventListener("click", () => {
  clearTimeout(debounce);
  stopWorker();
  revision++;
  notice.hidden = true;
  clearAnalysis("Analysis cancelled. Select Analyze to try again.");
});
document.getElementById("close-finding").addEventListener("click", () => { closeFinding(); editor.view.focus(); });
document.getElementById("previous-finding").addEventListener("click", () => stepFinding(-1));
document.getElementById("next-finding").addEventListener("click", () => stepFinding(1));
document.getElementById("inspect-finding").addEventListener("click", () => inspect(editor.view.state.selection.main.head, true));
document.getElementById("jump-to-finding").addEventListener("click", () => {
  const item = detailItems[detailIndex];
  if (item) editor.jump(item.start, item.end);
});
chartToggle.addEventListener("click", () => {
  const open = chartPanel.dataset.open !== "true";
  setDrawer(chartPanel, open);
  requestChart();
});
document.getElementById("close-chart").addEventListener("click", () => {
  setDrawer(chartPanel, false);
  chartToggle.focus();
});
for (const panel of [findingPanel, chartPanel]) panel.addEventListener("transitionend", () => scheduleViewport());
for (const control of document.querySelectorAll(".sample-choice")) control.addEventListener("click", () => {
  const sample = samples[control.dataset.sample];
  clearSampleSelection();
  control.setAttribute("aria-pressed", "true");
  filename = sample.format === "fountain" ? "scene.fountain" : "passage.md";
  replaceDocument(sample.text, sample.format);
});
document.getElementById("open-file").addEventListener("click", () => fileInput.click());
fileInput.addEventListener("change", () => {
  void openFile(fileInput.files?.[0]);
  fileInput.value = "";
});
document.getElementById("save-file").addEventListener("click", () => {
  const url = URL.createObjectURL(new Blob([editor.text()], { type: "text/plain;charset=utf-8" }));
  const link = document.createElement("a");
  link.href = url;
  link.download = filename;
  link.click();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
});
editorWrap.addEventListener("dragover", (event) => {
  if (!event.dataTransfer?.types.includes("Files")) return;
  event.preventDefault();
  dropHint.hidden = false;
});
editorWrap.addEventListener("dragleave", (event) => {
  if (!editorWrap.contains(event.relatedTarget)) dropHint.hidden = true;
});
editorWrap.addEventListener("drop", (event) => {
  if (!event.dataTransfer?.files.length) return;
  event.preventDefault();
  event.stopPropagation();
  dropHint.hidden = true;
  if (event.dataTransfer.files.length !== 1) { showError("Open one text file at a time."); return; }
  void openFile(event.dataTransfer.files[0]);
}, true);
document.addEventListener("click", (event) => {
  for (const { details } of toggleElements) if (!details.contains(event.target)) details.open = false;
});
document.addEventListener("keydown", (event) => {
  if (event.key !== "Escape") return;
  for (const { details } of toggleElements) details.open = false;
});
window.addEventListener("resize", () => {
  if (findingPanel.dataset.open === "true" && chartPanel.dataset.open === "true" && matchMedia("(max-width: 780px), (max-height: 650px)").matches) {
    setDrawer(lastDrawer === findingPanel ? chartPanel : findingPanel, false);
  }
  scheduleViewport();
});
window.addEventListener("beforeunload", (event) => {
  if (!dirty) return;
  event.preventDefault();
  event.returnValue = "";
});
run();
