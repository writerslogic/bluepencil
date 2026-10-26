import { after, afterEach, before, beforeEach, test } from "node:test";
import assert from "node:assert/strict";
import { chromium, webkit } from "playwright";
import { serve } from "./server.mjs";

let server;
let browser;
let context;
let page;
let errors;
const engine = process.env.BLUEPENCIL_BROWSER === "webkit" ? webkit : chromium;

async function ready(words) {
  await page.waitForFunction((expected) => {
    const stats = document.getElementById("stats").textContent;
    return stats.includes("words") && (!expected || stats.startsWith(expected)) && document.getElementById("viewport-summary").textContent.includes("near this passage");
  }, words === undefined ? "" : `${words.toLocaleString("en")} words`, { timeout: 60_000 });
}

async function open(text, words, name = "manuscript.md") {
  await page.locator("#file-input").setInputFiles({ name, mimeType: "text/plain", buffer: Buffer.from(text) });
  await ready(words);
}

before(async () => {
  server = await serve();
  browser = await engine.launch({ headless: true });
});
after(async () => { await browser?.close(); await server?.close(); });
beforeEach(async () => {
  context = await browser.newContext({ viewport: { width: 1440, height: 1000 }, acceptDownloads: true });
  page = await context.newPage();
  page.setDefaultTimeout(15_000);
  errors = [];
  page.on("pageerror", (error) => errors.push(error.message));
  page.on("console", (message) => { if (message.type() === "error") errors.push(message.text()); });
  await page.goto(server.url);
  await ready(124);
});
afterEach(async () => { await context.close(); assert.deepEqual(errors, []); });

test("inline colors open details, preserve editing, and support keyboard inspection", async () => {
  assert.equal(await page.locator("#findings").count(), 0);
  assert.equal(await page.locator(".finding-group").count(), 5);
  const echo = page.locator('.cm-finding[data-rule="echo"]').first();
  await echo.click();
  await page.waitForFunction(() => document.getElementById("finding-panel").dataset.open === "true");
  assert.match(await page.locator("#finding-explanation").textContent(), /word|phrase|narration/i);
  await page.locator("#close-finding").click();
  const content = page.locator(".cm-content");
  await content.focus();
  await page.keyboard.press("ControlOrMeta+A");
  await page.keyboard.insertText("🌊 She moved very slowly.");
  await ready(4);
  await page.locator("#inspect-finding").click();
  assert.equal(await page.locator("#finding-panel").getAttribute("data-open"), "true");
  await page.locator("#jump-to-finding").click();
  const selection = await page.evaluate(() => getSelection().toString());
  assert.ok(selection === "very" || selection === "slowly");
  await page.keyboard.press("Escape");
  assert.equal(await page.locator("#finding-panel").getAttribute("data-open"), "false");
  await page.keyboard.press("Alt+Enter");
  assert.equal(await page.locator("#finding-panel").getAttribute("data-open"), "true");
});

test("type switches only remove the chosen highlights", async () => {
  await page.locator('[data-group="style"] > summary').click();
  await page.locator('input[data-rule="hedge"]').uncheck();
  await page.locator("#desk-title").click();
  await page.waitForFunction(() => !document.querySelector('.cm-finding[data-rule="hedge"]'));
  assert.ok(await page.locator('.cm-finding[data-rule="echo"]').count() > 0);
  await page.locator('[data-group="style"] > summary').click();
  await page.locator('input[data-rule="hedge"]').check();
  await page.locator("#desk-title").click();
  await page.waitForFunction(() => document.querySelector('.cm-finding[data-rule="hedge"]'));
});

test("a complete manuscript stays responsive and scrolls to findings beyond the old limit", { timeout: 90_000 }, async () => {
  const body = "The lantern stood beside the window. She crossed the silent room.\n\n".repeat(17_000);
  const text = `# Arrival\n\nThe news came out of the blue.\n\n${body}# Departure\n\nShe had to make a decision before dawn.`;
  assert.ok(Buffer.byteLength(text) > 1_000_000);
  await open(text, 187_015);
  assert.equal(await page.locator('.cm-finding[data-rule="nominal"]').count(), 0);
  await page.locator('.cm-finding[data-rule="cliche"]').first().click();
  const pinned = await page.locator("#finding-heading").textContent();
  await page.locator(".cm-scroller").evaluate((element) => { element.scrollTop = element.scrollHeight; });
  await page.waitForFunction(() => document.querySelector('.cm-finding[data-rule="nominal"]'));
  assert.equal(await page.locator('.cm-finding[data-rule="cliche"]').count(), 0);
  assert.equal(await page.locator("#finding-heading").textContent(), pinned);
  await page.locator('.cm-finding[data-rule="nominal"]').click();
  assert.match(await page.locator("#finding-heading").textContent(), /make a decision/);
  assert.match(await page.locator("#finding-category").textContent(), /line 34,/);
  assert.ok(await page.locator(".cm-line").count() < 200);
});

test("chart lines toggle, follow scrolling when zoomed, and navigate back into the text", async () => {
  const paragraph = "She moved very slowly through the hall. The lantern stood beside the window.\n\n";
  const text = paragraph.repeat(400);
  await open(text, 5200);
  await page.locator("#toggle-chart").click();
  await page.waitForFunction(() => document.querySelectorAll(".chart-line").length === 3);
  const whole = await page.locator('.chart-line[data-series="sentence_length"]').getAttribute("d");
  await page.locator('[data-series="paragraph_length"][aria-pressed]').click();
  assert.equal(await page.locator('.chart-line[data-series="paragraph_length"]').count(), 0);
  await page.locator("#chart-scope").selectOption("visible");
  await page.waitForFunction((before) => document.querySelector('.chart-line[data-series="sentence_length"]')?.getAttribute("d") !== before, whole);
  const beforeScroll = await page.locator("#chart-tooltip").textContent();
  await page.locator(".cm-scroller").evaluate((element) => { element.scrollTop = element.scrollHeight * .65; });
  await page.waitForFunction((before) => document.getElementById("chart-tooltip").textContent !== before, beforeScroll);
  await page.locator("#chart-scope").selectOption("whole");
  await page.waitForFunction(() => document.querySelector(".chart-band"));
  await page.locator("#chart-cursor").focus();
  await page.locator("#chart-cursor").press("Home");
  await page.locator("#chart-jump").click();
  await page.waitForFunction(() => document.querySelector(".cm-scroller").scrollTop < 1000);
  assert.doesNotMatch(await page.locator("#analysis-chart").innerHTML(), /NaN|Infinity/);
});

test("pasting a multi-megabyte manuscript analyzes its ending", { timeout: 90_000 }, async () => {
  const body = "The lantern stood beside the window. She crossed the silent room.\n\n".repeat(34_000);
  const text = `${body}She had to make a decision before dawn.`;
  assert.ok(Buffer.byteLength(text) > 2_000_000);
  await page.locator(".cm-content").focus();
  await page.keyboard.press("ControlOrMeta+A");
  await page.evaluate((text) => {
    const data = new DataTransfer();
    data.setData("text/plain", text);
    document.querySelector(".cm-content").dispatchEvent(new ClipboardEvent("paste", { clipboardData: data, bubbles: true, cancelable: true }));
  }, text);
  await ready(374_008);
  await page.locator(".cm-scroller").evaluate((element) => { element.scrollTop = element.scrollHeight; });
  await page.waitForFunction(() => document.querySelector('.cm-finding[data-rule="nominal"]'));
  await page.locator('.cm-finding[data-rule="nominal"]').click();
  assert.match(await page.locator("#finding-heading").textContent(), /make a decision/);
});

test("viewport coordinates follow centered text as the drawers resize it", async () => {
  for (const paragraphs of [100, 8]) {
    const text = Array.from({ length: paragraphs }, (_, index) => `Paragraph ${index + 1}. She moved very slowly.`).join("\n\n");
    await open(text, paragraphs * 5);
    await page.locator('.cm-finding[data-rule="hedge"]').first().click();
    if (await page.locator("#chart-panel").getAttribute("data-open") !== "true") await page.locator("#toggle-chart").click();
    await page.waitForFunction(() => document.querySelector(".chart-line"));
    for (const ratio of [0, .45]) {
      await page.locator(".cm-scroller").evaluate((element, ratio) => { element.scrollTop = element.scrollHeight * ratio; }, ratio);
      await page.waitForFunction(() => {
        const viewport = document.querySelector(".cm-scroller").getBoundingClientRect();
        const visible = [...document.querySelectorAll(".cm-line")].flatMap((line) => {
          const rect = line.getBoundingClientRect();
          const match = line.textContent.match(/^Paragraph (\d+)/);
          return match && rect.bottom > viewport.top && rect.top < viewport.bottom ? [Number(match[1]) * 2 - 1] : [];
        });
        const shown = document.getElementById("viewport-summary").textContent.match(/lines ([\d,]+)–([\d,]+)/);
        if (!shown || !visible.length) return false;
        const first = Number(shown[1].replaceAll(",", ""));
        const last = Number(shown[2].replaceAll(",", ""));
        return Math.abs(first - Math.min(...visible)) <= 2 && Math.abs(last - Math.max(...visible)) <= 2;
      });
    }
  }
});

test("a manuscript at the 16 MiB boundary retains its ending and chart", { timeout: 120_000 }, async () => {
  const bytes = 16 * 1024 * 1024;
  const line = "The lantern stood beside the window. She crossed the silent room.\n\n";
  const ending = "\n\n# Departure\n\nShe had to make a decision before dawn.";
  const repeats = Math.floor((bytes - ending.length) / line.length);
  const text = line.repeat(repeats) + " ".repeat(bytes - ending.length - repeats * line.length) + ending;
  assert.equal(Buffer.byteLength(text), bytes);
  await open(text, repeats * 11 + 8);
  await page.locator(".cm-scroller").evaluate((element) => { element.scrollTop = element.scrollHeight; });
  await page.waitForFunction(() => document.querySelector('.cm-finding[data-rule="nominal"]'));
  await page.locator("#toggle-chart").click();
  await page.waitForFunction(() => document.querySelectorAll(".chart-line").length === 3);
  assert.ok(await page.locator(".chart-node").count() <= 750);
});

test("cancellation and edits cannot publish stale manuscript results", async () => {
  const text = "The lantern stood beside the window. She crossed the silent room.\n\n".repeat(17_000);
  await page.locator("#file-input").setInputFiles({ name: "novel.txt", mimeType: "text/plain", buffer: Buffer.from(text) });
  await page.locator("#cancel-analysis").waitFor({ state: "visible" });
  await page.locator("#cancel-analysis").click();
  assert.match(await page.locator("#viewport-summary").textContent(), /cancelled/);
  await page.locator(".cm-content").focus();
  await page.keyboard.press("ControlOrMeta+A");
  await page.keyboard.insertText("She moved very slowly.");
  await ready(4);
  assert.match(await page.locator("#stats").textContent(), /^4 words/);
  assert.equal(await page.locator('.cm-finding[data-rule="nominal"]').count(), 0);
});

test("mobile import errors remain visible and charts fit the editor", async () => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.locator("#file-input").setInputFiles({ name: "too-large.txt", mimeType: "text/plain", buffer: Buffer.alloc(16 * 1024 * 1024 + 1, 97) });
  assert.equal(await page.locator("#analysis-notice").isVisible(), true);
  assert.match(await page.locator("#analysis-message").textContent(), /16 MiB/);
  await page.locator("#file-input").setInputFiles({ name: "invalid.txt", mimeType: "text/plain", buffer: Buffer.from([255]) });
  await page.waitForFunction(() => document.getElementById("analysis-message").textContent.includes("UTF-8"));
  assert.match(await page.locator("#stats").textContent(), /^124 words/);
  await page.locator("#toggle-chart").click();
  await page.waitForFunction(() => document.querySelector(".chart-line"));
  const overflow = await page.evaluate(() => document.documentElement.scrollWidth > innerWidth);
  assert.equal(overflow, false);
  assert.ok(await page.locator(".cm-scroller").evaluate((element) => element.clientHeight) > 150);
  for (const size of [{ width: 320, height: 568 }, { width: 844, height: 390 }]) {
    await page.setViewportSize(size);
    await page.waitForFunction(() => document.getElementById("chart-panel").getBoundingClientRect().bottom <= document.querySelector(".writing-desk").getBoundingClientRect().bottom + 2);
    assert.ok(await page.locator(".cm-scroller").evaluate((element) => element.clientHeight) >= 70);
  }
});

test("download preserves the document and replacing a sample can be undone", async () => {
  const download = page.waitForEvent("download");
  await page.locator("#save-file").click();
  const file = await download;
  const stream = await file.createReadStream();
  const chunks = [];
  for await (const chunk of stream) chunks.push(chunk);
  assert.match(Buffer.concat(chunks).toString("utf8"), /Mara crossed the hotel lobby/);
  await page.locator('[data-sample="screenplay"]').click();
  await ready(86);
  await page.locator(".cm-content").focus();
  await page.keyboard.press("ControlOrMeta+Z");
  await page.locator("#format").selectOption("markdown");
  await ready(124);
});

test("editing sends no document requests", async () => {
  const requests = [];
  page.on("request", (request) => requests.push({ url: request.url(), method: request.method() }));
  await page.locator(".cm-content").focus();
  await page.keyboard.press("ControlOrMeta+A");
  await page.keyboard.insertText("She moved very slowly.");
  await ready(4);
  assert.ok(requests.every((request) => request.method === "GET" && request.url.startsWith(server.url)));
});

test("versioned assets avoid mixed builds and the legacy entry preserves an existing draft", async () => {
  assert.match(await page.locator('script[type="module"]').getAttribute("src"), /playground-[A-Z0-9]+\.js$/);
  assert.match(await page.locator('link[rel="stylesheet"]').getAttribute("href"), /style-[a-f0-9]+\.css$/);
  const legacy = await context.newPage();
  await legacy.route(`${server.url}/legacy.html`, (route) => route.fulfill({
    contentType: "text/html",
    body: '<textarea id="source">UNSAVED MANUSCRIPT</textarea><p id="status">Loading</p><script type="module" src="./playground.js"></script>',
  }));
  await legacy.goto(`${server.url}/legacy.html`);
  await legacy.getByRole("link", { name: "Open the updated editor" }).waitFor();
  assert.equal(await legacy.locator("#source").inputValue(), "UNSAVED MANUSCRIPT");
  assert.equal(new URL(legacy.url()).pathname, "/legacy.html");
  await legacy.close();
});
