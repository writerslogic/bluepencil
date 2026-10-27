export const series = [
  { id: "sentence_length", label: "Sentence length", color: "#9bbfff", unit: "words/sentence" },
  { id: "paragraph_length", label: "Paragraph length", color: "#7dcfb2", unit: "words/paragraph" },
  { id: "dialogue_share", label: "Dialogue share", color: "#e2ba75", unit: "% dialogue" },
  { id: "finding_density", label: "Finding density", color: "#e3a3c5", unit: "findings/1k words" },
  { id: "sentence_variation", label: "Sentence variation", color: "#c5a4f4", unit: "% · last 5 sentences" },
];

const namespace = "http://www.w3.org/2000/svg";
const number = (value) => value.toLocaleString("en", { maximumFractionDigits: 1 });
function element(name, attributes) {
  const node = document.createElementNS(namespace, name);
  for (const [key, value] of Object.entries(attributes)) node.setAttribute(key, String(value));
  return node;
}
function textElement(name, className, text) {
  const node = document.createElement(name);
  node.className = className;
  node.textContent = text;
  return node;
}
function extent(points, metric) {
  if (metric === "dialogue_share") return [0, 100];
  let low = Infinity;
  let high = 0;
  for (const point of points) { low = Math.min(low, point[metric]); high = Math.max(high, point[metric]); }
  if (!Number.isFinite(low) || high === 0) return [0, 1];
  const padding = Math.max((high - low) * .12, high * .05, .1);
  return [metric.endsWith("length") ? Math.max(0, low - padding) : 0, high + padding];
}

export function createChart({ request, jump }) {
  const svg = document.getElementById("analysis-chart");
  const content = document.querySelector(".chart-content");
  const plot = document.getElementById("chart-plot");
  const controls = document.getElementById("chart-series");
  const layoutControls = document.getElementById("chart-layout");
  const labels = document.getElementById("chart-track-labels");
  const scales = document.getElementById("chart-track-scales");
  const scope = document.getElementById("chart-scope");
  const detail = document.getElementById("chart-detail");
  const resolution = document.getElementById("chart-resolution");
  const chapters = document.getElementById("chart-chapters");
  const axisStart = document.getElementById("chart-axis-start");
  const axisEnd = document.getElementById("chart-axis-end");
  const empty = document.getElementById("chart-empty");
  const cursor = document.getElementById("chart-cursor");
  const tooltip = document.getElementById("chart-tooltip");
  const jumpButton = document.getElementById("chart-jump");
  const note = document.getElementById("chart-note");
  const active = new Set(["sentence_length", "paragraph_length", "dialogue_share"]);
  const choices = { whole: "auto", visible: "auto" };
  const values = new Map();
  const legendValues = new Map();
  const coordinates = new Map();
  let data;
  let range;
  let chapterCount = 0;
  let selected = 0;
  let layout = "tracks";
  let width = 1;
  let height = 1;
  let loading = false;
  let failure = "";

  function configureDetail() {
    const options = scope.value === "visible"
      ? [["auto", "Auto · every sentence"], ["sentence", "Every sentence"], ["paragraph", "Every paragraph"]]
      : [["auto", chapterCount >= 2 && chapterCount <= 1024 ? "Auto · 8 / chapter" : "Auto · 256 samples"],
        ...(chapterCount >= 2 ? [["chapter-4", "4 samples / chapter"], ["chapter-8", "8 samples / chapter"], ["chapter-16", "16 samples / chapter"]] : []),
        ["overview-128", "128 overview samples"], ["overview-256", "256 overview samples"], ["overview-512", "512 overview samples"]];
    detail.replaceChildren(...options.map(([value, label]) => new Option(label, value)));
    if (!options.some(([value]) => value === choices[scope.value])) choices[scope.value] = "auto";
    detail.value = choices[scope.value];
  }

  function x(position) {
    const from = data?.from || 0;
    const to = data?.to || from + 1;
    return 4 + Math.max(0, Math.min(1, (position - from) / (to - from || 1))) * Math.max(1, width - 8);
  }

  function showPoint(index) {
    if (!data?.points.length) return;
    selected = Math.max(0, Math.min(index, data.points.length - 1));
    cursor.value = String(selected);
    const point = data.points[selected];
    const section = data.sections.find((section) => section.id === point.section);
    const sentence = point.first_sentence === point.last_sentence ? `Sentence ${number(point.first_sentence)}` : `Sentences ${number(point.first_sentence)}–${number(point.last_sentence)}`;
    const paragraph = point.first_paragraph === point.last_paragraph ? `paragraph ${number(point.first_paragraph)}` : `paragraphs ${number(point.first_paragraph)}–${number(point.last_paragraph)}`;
    tooltip.textContent = `${section ? `${section.title} · ` : ""}${sentence} · ${paragraph} · words ${number(point.first_word)}–${number(point.last_word)}`;
    const spokenValues = [];
    for (const item of series) {
      const value = number(point[item.id]);
      legendValues.get(item.id).textContent = value;
      legendValues.get(item.id).title = `${value} ${item.unit}`;
      if (values.has(item.id)) values.get(item.id).firstChild.textContent = value;
      if (active.has(item.id)) spokenValues.push(`${item.label}: ${value} ${item.unit}`);
      const marker = svg.querySelector(`.chart-selected-node[data-series="${item.id}"]`);
      const coordinate = coordinates.get(item.id)?.[selected];
      if (marker && coordinate) { marker.setAttribute("cx", coordinate.x); marker.setAttribute("cy", coordinate.y); }
    }
    cursor.setAttribute("aria-valuetext", `${tooltip.textContent}. ${spokenValues.join("; ")}`);
    const line = svg.querySelector(".chart-cursor");
    const at = x((point.from + point.to) / 2);
    line?.setAttribute("x1", at);
    line?.setAttribute("x2", at);
  }

  function drawBand() {
    svg.querySelector(".chart-viewport")?.remove();
    if (!data?.points.length || !range) return;
    const group = element("g", { class: "chart-viewport" });
    for (const [from, to, className] of [[range.from, range.to, "chart-band"], [range.visibleFrom, range.visibleTo, "chart-visible"]]) {
      if (!Number.isFinite(from) || !Number.isFinite(to) || to < data.from || from > data.to) continue;
      const left = x(from);
      group.append(element("rect", { class: className, x: left, y: 0, width: Math.max(2, x(to) - left), height }));
    }
    svg.prepend(group);
  }

  function drawChapters() {
    chapters.replaceChildren();
    if (!data?.points.length) return;
    let lastLabel = 75;
    for (const section of data.sections) {
      const left = x(section.from);
      const right = x(section.to);
      if (section.id % 2 === 1) svg.append(element("rect", { class: "chart-section-fill", x: left, y: 0, width: right - left, height }));
      if (left > 5) svg.append(element("line", { class: "chart-section-line", x1: left, x2: left, y1: 0, y2: height }));
      const middle = (left + right) / 2;
      if (middle < lastLabel || middle > width - 75) continue;
      const label = textElement("span", "chart-chapter-label", section.title);
      const labelWidth = Math.min(150, Math.max(60, right - left - 8));
      label.style.left = `${middle / width * 100}%`;
      label.style.width = `${labelWidth}px`;
      label.title = section.title;
      chapters.append(label);
      lastLabel = middle + labelWidth / 2 + 35;
    }
    axisStart.textContent = `Word ${number(data.points[0].first_word)}`;
    axisEnd.textContent = number(data.points.at(-1).last_word);
  }

  function render() {
    svg.replaceChildren();
    labels.replaceChildren();
    scales.replaceChildren();
    chapters.replaceChildren();
    values.clear();
    coordinates.clear();
    axisStart.textContent = "";
    axisEnd.textContent = "";
    const points = data?.points || [];
    const enabled = series.filter((item) => active.has(item.id));
    content.dataset.layout = layout;
    plot.dataset.layout = layout;
    plot.style.setProperty("--chart-tracks", Math.max(1, enabled.length));
    const rect = svg.getBoundingClientRect();
    width = Math.max(20, rect.width);
    height = Math.max(20, rect.height);
    svg.setAttribute("viewBox", `0 0 ${width} ${height}`);
    svg.dataset.points = points.length;
    svg.dataset.resolution = data?.resolution || "";
    empty.hidden = points.length > 0 && enabled.length > 0;
    empty.textContent = failure || (points.length ? "Choose a metric above to explore this passage." : "Analyze a manuscript to explore its shape.");
    empty.dataset.kind = failure ? "error" : "empty";
    cursor.disabled = loading || !points.length || !enabled.length;
    jumpButton.disabled = loading || !points.length || !enabled.length;
    cursor.max = String(Math.max(0, points.length - 1));
    if (!points.length) { tooltip.textContent = ""; return; }
    drawChapters();
    const rowHeight = height / Math.max(1, enabled.length);
    for (const [index, item] of enabled.entries()) {
      const top = layout === "tracks" ? index * rowHeight : 0;
      const bottom = layout === "tracks" ? top + rowHeight : height;
      const inset = Math.min(7, (bottom - top) / 5);
      const [low, high] = extent(points, item.id);
      const coords = points.map((point) => ({ x: x((point.from + point.to) / 2), y: bottom - inset - (point[item.id] - low) / (high - low) * (bottom - top - inset * 2) }));
      coordinates.set(item.id, coords);
      if (layout === "tracks") {
        const label = textElement("div", "chart-track-label", "");
        label.style.setProperty("--series-color", item.color);
        label.style.top = `${index / enabled.length * 100}%`;
        label.style.height = `${100 / enabled.length}%`;
        const value = textElement("span", "chart-track-value", "0");
        value.append(textElement("small", "", item.unit));
        label.append(textElement("span", "chart-track-name", item.label), value);
        labels.append(label);
        values.set(item.id, value);
        const scale = textElement("div", "chart-track-scale", "");
        scale.style.top = label.style.top;
        scale.style.height = label.style.height;
        scale.append(textElement("span", "", number(high)), textElement("span", "", number(low)));
        scales.append(scale);
        svg.append(element("line", { class: "chart-grid", x1: 4, x2: width - 4, y1: top + rowHeight / 2, y2: top + rowHeight / 2 }));
        if (index) svg.append(element("line", { class: "chart-row-divider", x1: 0, x2: width, y1: top, y2: top }));
      }
      const pathData = coords.map((point, index) => `${index ? "L" : "M"}${point.x.toFixed(2)},${point.y.toFixed(2)}`).join(" ");
      if (layout === "tracks") svg.append(element("path", { class: "chart-area", fill: item.color, d: `${pathData} L${coords.at(-1).x},${bottom - inset} L${coords[0].x},${bottom - inset} Z` }));
      const path = element("path", { class: "chart-line", "data-series": item.id, stroke: item.color, d: pathData });
      const title = element("title", {});
      title.textContent = `${item.label}, ${item.unit}. Scale ${number(low)} to ${number(high)}.`;
      path.append(title);
      svg.append(path);
      let lastNode = -Infinity;
      for (const [index, point] of coords.entries()) {
        if (point.x - lastNode < 4 && index !== coords.length - 1) continue;
        svg.append(element("circle", { class: "chart-node", cx: point.x, cy: point.y, r: 2, fill: item.color }));
        lastNode = point.x;
      }
      svg.append(element("circle", { class: "chart-selected-node", "data-series": item.id, r: 3.2, fill: item.color }));
    }
    svg.append(element("line", { class: "chart-cursor", x1: 4, x2: 4, y1: 0, y2: height }));
    note.textContent = `${layout === "tracks" ? "Each track has its own labeled scale" : "Overlay uses independent relative scales"} · shading follows the visible passage + buffer · variation uses up to 5 sentences. Select a node to go to the text.`;
    drawBand();
    showPoint(selected);
  }

  for (const item of series) {
    const button = document.createElement("button");
    button.type = "button";
    button.dataset.series = item.id;
    button.style.setProperty("--series-color", item.color);
    button.setAttribute("aria-pressed", String(active.has(item.id)));
    const swatch = textElement("span", "swatch", "");
    swatch.setAttribute("aria-hidden", "true");
    const value = textElement("span", "chart-series-value", "");
    value.setAttribute("aria-hidden", "true");
    legendValues.set(item.id, value);
    button.append(swatch, document.createTextNode(item.label), value);
    button.addEventListener("click", () => {
      if (active.has(item.id)) active.delete(item.id); else active.add(item.id);
      button.setAttribute("aria-pressed", String(active.has(item.id)));
      render();
    });
    controls.append(button);
  }
  for (const button of layoutControls.querySelectorAll("button")) button.addEventListener("click", () => {
    layout = button.dataset.layout;
    for (const option of layoutControls.querySelectorAll("button")) option.setAttribute("aria-pressed", String(option === button));
    render();
  });
  scope.addEventListener("change", () => { configureDetail(); request(); });
  detail.addEventListener("change", () => { choices[scope.value] = detail.value; request(); });
  cursor.addEventListener("input", () => showPoint(Number(cursor.value)));
  jumpButton.addEventListener("click", () => {
    const point = data?.points[selected];
    if (point) jump(point.from);
  });
  function pointAt(event) {
    if (!data?.points.length) return 0;
    const at = event.clientX - svg.getBoundingClientRect().left;
    let nearest = 0;
    let distance = Infinity;
    for (let i = 0; i < data.points.length; i++) {
      const point = data.points[i];
      const delta = Math.abs(x((point.from + point.to) / 2) - at);
      if (delta < distance) { distance = delta; nearest = i; }
    }
    return nearest;
  }
  svg.addEventListener("pointermove", (event) => { if (!loading) showPoint(pointAt(event)); });
  svg.addEventListener("click", (event) => {
    if (loading || !active.size) return;
    showPoint(pointAt(event));
    const point = data?.points[selected];
    if (point) jump(point.from);
  });
  let previousWidth = 0;
  let previousHeight = 0;
  new ResizeObserver(([entry]) => {
    if (entry.contentRect.width === previousWidth && entry.contentRect.height === previousHeight) return;
    previousWidth = entry.contentRect.width;
    previousHeight = entry.contentRect.height;
    render();
  }).observe(svg);
  configureDetail();
  render();
  return {
    scope: () => scope.value,
    detail: () => detail.value,
    describe(summary) { chapterCount = summary.chapters; configureDetail(); },
    loading() { loading = true; failure = ""; content.classList.add("is-loading"); resolution.textContent = "Updating samples…"; cursor.disabled = true; jumpButton.disabled = true; },
    error(message) { loading = false; data = undefined; failure = message; content.classList.remove("is-loading"); resolution.textContent = "Choose another detail setting"; render(); },
    update(result) {
      const position = data?.points[selected]?.from ?? range?.visibleFrom ?? result.from;
      data = result;
      selected = 0;
      let distance = Infinity;
      for (const [index, point] of result.points.entries()) {
        const delta = Math.abs(point.from - position);
        if (delta < distance) { selected = index; distance = delta; }
      }
      loading = false;
      failure = "";
      content.classList.remove("is-loading");
      const unit = result.resolution === "chapter" ? `up to ${result.target} samples / chapter` : result.resolution === "sentence" ? "every sentence" : result.resolution === "paragraph" ? "every paragraph" : `${result.target}-sample overview`;
      resolution.textContent = `${number(result.points.length)} nodes / line · ${unit}`;
      render();
    },
    viewport(value) { range = value; drawBand(); },
    clear() { data = undefined; loading = false; failure = ""; content.classList.remove("is-loading"); resolution.textContent = "Explore the shape of your text"; render(); },
  };
}
