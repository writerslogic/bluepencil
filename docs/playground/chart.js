export const series = [
  { id: "sentence_length", label: "Sentence length", color: "#9bbfff", unit: "words/sentence" },
  { id: "paragraph_length", label: "Paragraph length", color: "#7dcfb2", unit: "words/paragraph" },
  { id: "dialogue_share", label: "Dialogue share", color: "#e2ba75", unit: "% dialogue" },
  { id: "finding_density", label: "Finding density", color: "#e3a3c5", unit: "observations/1k words" },
  { id: "sentence_variation", label: "Sentence variation", color: "#c5a4f4", unit: "% variation" },
];

const namespace = "http://www.w3.org/2000/svg";
function element(name, attributes) {
  const node = document.createElementNS(namespace, name);
  for (const [key, value] of Object.entries(attributes)) node.setAttribute(key, String(value));
  return node;
}

export function createChart({ request, jump }) {
  const svg = document.getElementById("analysis-chart");
  const controls = document.getElementById("chart-series");
  const scope = document.getElementById("chart-scope");
  const interval = document.getElementById("chart-interval");
  const empty = document.getElementById("chart-empty");
  const cursor = document.getElementById("chart-cursor");
  const tooltip = document.getElementById("chart-tooltip");
  const jumpButton = document.getElementById("chart-jump");
  const note = document.getElementById("chart-note");
  const active = new Set(["sentence_length", "paragraph_length", "dialogue_share"]);
  let data;
  let range;
  let selected = 0;

  function x(position) {
    const first = data.points[0]?.from || 0;
    const last = data.points.at(-1)?.to || first + 1;
    return 12 + Math.max(0, Math.min(1, (position - first) / (last - first || 1))) * 976;
  }

  function showPoint(index) {
    if (!data?.points.length) return;
    selected = Math.max(0, Math.min(index, data.points.length - 1));
    cursor.value = String(selected);
    const point = data.points[selected];
    const values = series.filter((item) => active.has(item.id)).map((item) => `${point[item.id].toFixed(1)} ${item.unit}`);
    tooltip.textContent = `Words ${point.first_word.toLocaleString("en")}–${point.last_word.toLocaleString("en")} · ${values.join(" · ")}`;
    cursor.setAttribute("aria-valuetext", tooltip.textContent);
    const line = svg.querySelector(".chart-cursor");
    const at = x((point.from + point.to) / 2);
    line?.setAttribute("x1", String(at));
    line?.setAttribute("x2", String(at));
  }

  function drawBand() {
    svg.querySelector(".chart-band")?.remove();
    if (!data?.points.length || !range || scope.value !== "whole") return;
    const left = x(range.from);
    const right = x(range.to);
    const band = element("rect", { class: "chart-band", x: left, y: 8, width: Math.max(2, right - left), height: 144 });
    const title = element("title", {});
    title.textContent = "Visible passage and surrounding buffer";
    band.append(title);
    svg.prepend(band);
  }

  function render() {
    svg.replaceChildren();
    const points = data?.points || [];
    empty.hidden = points.length > 0 && active.size > 0;
    empty.textContent = points.length ? "Choose a line above to explore this passage." : "Analyze a manuscript to explore its shape.";
    cursor.disabled = !points.length;
    jumpButton.disabled = !points.length;
    cursor.max = String(Math.max(0, points.length - 1));
    for (const y of [12, 57, 102, 147]) svg.append(element("line", { class: "chart-grid", x1: 12, x2: 988, y1: y, y2: y }));
    if (!points.length) {
      tooltip.textContent = "";
      return;
    }
    for (const item of series) {
      if (!active.has(item.id)) continue;
      let minimum = Infinity;
      let maximum = -Infinity;
      for (const point of points) {
        minimum = Math.min(minimum, point[item.id]);
        maximum = Math.max(maximum, point[item.id]);
      }
      const coords = points.map((point) => {
        const value = maximum === minimum ? (maximum === 0 ? 0 : .5) : (point[item.id] - minimum) / (maximum - minimum);
        return { x: x((point.from + point.to) / 2), y: 147 - value * 135 };
      });
      const path = element("path", { class: "chart-line", "data-series": item.id, stroke: item.color, d: coords.map((point, index) => `${index ? "L" : "M"}${point.x.toFixed(2)},${point.y.toFixed(2)}`).join(" ") });
      const title = element("title", {});
      title.textContent = `${item.label}, ${item.unit}; independently scaled in this view`;
      path.append(title);
      svg.append(path);
      for (const point of coords) svg.append(element("circle", { class: "chart-node", cx: point.x, cy: point.y, r: 2.2, fill: item.color }));
    }
    svg.append(element("line", { class: "chart-cursor", x1: 12, x2: 12, y1: 8, y2: 152 }));
    note.textContent = `Each line has its own relative scale · ≈${data.interval.toLocaleString("en")} words per point · ${scope.value === "whole" ? "shading follows the visible passage" : "following the visible passage and buffer"}. These are pacing signals, not a pacing score.`;
    drawBand();
    showPoint(selected);
  }

  for (const item of series) {
    const button = document.createElement("button");
    button.type = "button";
    button.dataset.series = item.id;
    button.style.setProperty("--series-color", item.color);
    button.setAttribute("aria-pressed", String(active.has(item.id)));
    const swatch = document.createElement("span");
    swatch.className = "swatch";
    swatch.setAttribute("aria-hidden", "true");
    button.append(swatch, document.createTextNode(item.label));
    button.addEventListener("click", () => {
      if (active.has(item.id)) active.delete(item.id); else active.add(item.id);
      button.setAttribute("aria-pressed", String(active.has(item.id)));
      render();
    });
    controls.append(button);
  }
  scope.addEventListener("change", request);
  interval.addEventListener("change", request);
  cursor.addEventListener("input", () => showPoint(Number(cursor.value)));
  jumpButton.addEventListener("click", () => {
    const point = data?.points[selected];
    if (point) jump(point.from);
  });
  function pointAt(event) {
    if (!data?.points.length) return 0;
    const rect = svg.getBoundingClientRect();
    const at = (event.clientX - rect.left) / rect.width * 1000;
    let nearest = 0;
    let distance = Infinity;
    for (let i = 0; i < data.points.length; i++) {
      const point = data.points[i];
      const delta = Math.abs(x((point.from + point.to) / 2) - at);
      if (delta < distance) { distance = delta; nearest = i; }
    }
    return nearest;
  }
  svg.addEventListener("pointermove", (event) => showPoint(pointAt(event)));
  svg.addEventListener("click", (event) => {
    showPoint(pointAt(event));
    const point = data?.points[selected];
    if (point) jump(point.from);
  });
  render();
  return {
    scope: () => scope.value,
    interval: () => Number(interval.value),
    update(result) { data = result; selected = Math.min(selected, Math.max(0, result.points.length - 1)); render(); },
    viewport(value) { range = value; drawBand(); },
    clear() { data = undefined; render(); },
  };
}
