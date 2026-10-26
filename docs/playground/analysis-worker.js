import init, { Manuscript } from "./pkg/bluepencil_web.js";

let manuscript;
let analysisId;

self.onmessage = async ({ data }) => {
  if (!Number.isSafeInteger(data?.id)) return;
  if (data.type === "visible" || data.type === "chart") {
    if (!manuscript || data.id !== analysisId || !Number.isSafeInteger(data.query)) return;
    try {
      if (!Number.isSafeInteger(data.from) || !Number.isSafeInteger(data.to) || data.from < 0 || data.to < data.from) throw new Error("Choose a passage within this manuscript.");
      if (data.type === "visible" && (!Array.isArray(data.enabled) || data.enabled.length > 15 || data.enabled.some((rule) => typeof rule !== "string"))) throw new Error("Choose available finding types.");
      if (data.type === "chart" && ![0, 25, 50, 250, 1000].includes(data.interval)) throw new Error("Choose an available chart interval.");
      const result = data.type === "visible"
        ? manuscript.visible(data.from, data.to, JSON.stringify(data.enabled))
        : manuscript.chart(data.from, data.to, data.interval);
      self.postMessage({ type: data.type, id: data.id, query: data.query, result: JSON.parse(result) });
    } catch (error) {
      self.postMessage({ type: "error", request: data.type, id: data.id, query: data.query, message: String(error) });
    }
    return;
  }
  if (data.type !== "analyze" || typeof data.text !== "string" || !["plain", "markdown", "fountain"].includes(data.format)) return;
  if (analysisId !== undefined) return;
  analysisId = data.id;
  try {
    self.postMessage({ type: "progress", id: data.id, message: "Reading your manuscript…", completed: 0, total: 7 });
    await init();
    manuscript?.free();
    manuscript = new Manuscript(data.text, data.format);
    const stages = manuscript.stages();
    for (let stage = 0; stage < stages; stage++) {
      self.postMessage({ type: "progress", id: data.id, message: manuscript.next_stage(), completed: stage + 1, total: stages + 1 });
      await new Promise((resolve) => setTimeout(resolve, 0));
      manuscript.advance();
    }
    self.postMessage({
      type: "complete",
      id: data.id,
      summary: JSON.parse(manuscript.summary()),
    });
  } catch (error) {
    manuscript?.free();
    manuscript = undefined;
    self.postMessage({ type: "error", id: data.id, message: String(error) });
  }
};
