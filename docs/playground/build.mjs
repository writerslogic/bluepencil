import { build } from "esbuild";
import { createHash } from "node:crypto";
import { copyFile, mkdir, readFile, readdir, writeFile } from "node:fs/promises";
import { basename, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL(".", import.meta.url));
const destination = resolve(root, process.argv[2] || "../book");
const digest = (...values) => {
  const hash = createHash("sha256");
  for (const value of values) hash.update(value);
  return hash.digest("hex").slice(0, 12);
};
const engineFiles = ["bluepencil_web.js", "bluepencil_web_bg.wasm"];
const engineHash = digest(...await Promise.all(engineFiles.map((file) => readFile(resolve(destination, "pkg", file)))));
const engineDirectory = resolve(destination, "pkg", engineHash);
await mkdir(engineDirectory, { recursive: true });
for (const file of [...engineFiles, "LICENSE"]) await copyFile(resolve(destination, "pkg", file), resolve(engineDirectory, file));

const shared = { absWorkingDir: root, bundle: true, format: "esm", target: "es2022", minify: true, outdir: destination, entryNames: "[name]-[hash]", metafile: true, legalComments: "linked" };
const worker = await build({
  ...shared,
  entryPoints: ["analysis-worker.js"],
  plugins: [{ name: "versioned-engine", setup(builder) {
    builder.onResolve({ filter: /^\.\/pkg\/bluepencil_web\.js$/ }, () => ({ path: `./pkg/${engineHash}/bluepencil_web.js`, external: true }));
  } }],
});
const entry = (result) => basename(Object.entries(result.metafile.outputs).find(([, output]) => output.entryPoint)[0]);
const workerFile = entry(worker);
const result = await build({
  ...shared,
  entryPoints: ["playground.js"],
  define: { BLUEPENCIL_WORKER_URL: JSON.stringify(`./${workerFile}`) },
  banner: { js: "/*! Third-party licenses: ./playground.js.LICENSE.txt */" },
});
const appFile = entry(result);
const style = await readFile(resolve(root, "style.css"));
const styleFile = `style-${digest(style)}.css`;
await writeFile(resolve(destination, styleFile), style);
const html = (await readFile(resolve(root, "index.html"), "utf8"))
  .replace('src="./playground.js"', `src="./${appFile}"`)
  .replace('href="./style.css"', `href="./${styleFile}"`);
await writeFile(resolve(destination, "index.html"), html);
await writeFile(resolve(destination, "playground.js"), `
const destination = new URL("./", import.meta.url);
destination.searchParams.set("v", ${JSON.stringify(appFile)});
if (document.getElementById("finding-toggles") && document.getElementById("chart-detail")) {
  await import(${JSON.stringify(`./${appFile}`)});
} else {
  const notice = document.getElementById("status") || document.createElement("p");
  const link = document.createElement("a");
  link.href = destination.href;
  link.textContent = "Open the updated editor";
  notice.replaceChildren(document.createTextNode("bluepencil has been updated. Your text is still here; copy it before opening the new editor. "), link);
  if (!notice.isConnected) document.body.prepend(notice);
}
`);

const packages = new Set();
for (const input of Object.keys(result.metafile.inputs)) {
  const parts = input.split("/");
  const index = parts.lastIndexOf("node_modules");
  if (index < 0) continue;
  const end = index + (parts[index + 1].startsWith("@") ? 3 : 2);
  packages.add(parts.slice(0, end).join("/"));
}
const notices = [];
for (const path of [...packages].sort()) {
  const directory = resolve(root, path);
  const file = (await readdir(directory)).find((name) => /^licen[cs]e(?:\..*)?$/i.test(name));
  if (!file) throw new Error(`Missing license for bundled package ${path}`);
  notices.push(`${path}\n\n${await readFile(resolve(directory, file), "utf8")}`);
}
await writeFile(resolve(destination, "playground.js.LICENSE.txt"), notices.join("\n\n"));
