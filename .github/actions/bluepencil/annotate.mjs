import { execFileSync, spawnSync } from "node:child_process";
import { appendFileSync } from "node:fs";

const sourceExt = /\.(?:md|markdown|mdown|mkd|mdx|fountain|spmd|txt)$/i;
const base = process.env.BLUEPENCIL_BASE_SHA || "";
const limit = Number(process.env.BLUEPENCIL_LIMIT || "25");
if (base && !/^[0-9a-f]{40,64}$/i.test(base)) throw new Error("base-sha must be a full commit SHA");
if (!Number.isInteger(limit) || limit < 1 || limit > 50) throw new Error("limit must be between 1 and 50");
const gitArgs = base
  ? ["diff", "--name-only", "-z", "--diff-filter=ACM", base, "HEAD", "--"]
  : ["ls-files", "-z", "--"];
const files = execFileSync("git", gitArgs).toString("utf8").split("\0").filter((file) => sourceExt.test(file));
if (files.length === 0) {
  console.log("No changed writing files to review.");
  process.exit(0);
}
if (files.length > 500) throw new Error("More than 500 writing files matched; split the review into smaller runs");
const cli = spawnSync("bluepencil", ["--json", "--limit", String(limit), "report", ...files.map((file) => `./${file}`)], {
  encoding: "utf8",
  maxBuffer: 32 * 1024 * 1024,
});
if (cli.error) throw cli.error;
if (cli.status !== 0) throw new Error(`bluepencil report failed: ${cli.stderr.trim()}`);
const output = JSON.parse(cli.stdout);
const observations = output.revisions || [];
const prop = (value) => String(value).replaceAll("%", "%25").replaceAll("\r", "%0D").replaceAll("\n", "%0A").replaceAll(":", "%3A").replaceAll(",", "%2C");
const msg = (value) => String(value).replaceAll("%", "%25").replaceAll("\r", "%0D").replaceAll("\n", "%0A");
for (const item of observations) {
  const file = item.file.replace(/^\.\//, "");
  console.log(`::notice file=${prop(file)},line=${item.line},col=${item.column},title=${prop(`bluepencil: ${item.rule}`)}::${msg(item.message)}`);
}
console.log(`bluepencil reviewed ${files.length} file(s) and found ${observations.length} observation(s).`);
if (process.env.GITHUB_STEP_SUMMARY) {
  appendFileSync(process.env.GITHUB_STEP_SUMMARY, `### bluepencil prose review\n\n${files.length} writing file(s) reviewed; ${observations.length} observations annotated. Style observations are prompts for the writer, not required corrections.\n`);
}
