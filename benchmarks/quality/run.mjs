import { readFileSync } from "node:fs";
import { spawnSync } from "node:child_process";

const cases = JSON.parse(readFileSync(new URL("./cases.json", import.meta.url)));
const binary = process.env.BLUEPENCIL_BIN || "bluepencil";
const scores = new Map();
for (const [index, item] of cases.entries()) {
  const result = spawnSync(binary, ["--json", item.rule, "-"], { input: item.text, encoding: "utf8" });
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error(`${item.rule} failed: ${result.stderr}`);
  const output = JSON.parse(result.stdout);
  const found = output.findings.length > 0;
  const score = scores.get(item.rule) || { tp: 0, fp: 0, tn: 0, fn: 0 };
  score[item.positive ? found ? "tp" : "fn" : found ? "fp" : "tn"]++;
  scores.set(item.rule, score);
  if (found !== item.positive) console.log(`Mismatch #${index + 1}: ${item.rule} ${JSON.stringify(item.text)} (expected ${item.positive}, found ${found})`);
}
let total = { tp: 0, fp: 0, tn: 0, fn: 0 };
for (const [rule, score] of scores) {
  for (const key of Object.keys(total)) total[key] += score[key];
  console.log(`${rule.padEnd(10)} TP ${score.tp}  FP ${score.fp}  TN ${score.tn}  FN ${score.fn}`);
}
const precision = total.tp + total.fp ? total.tp / (total.tp + total.fp) : 0;
const recall = total.tp + total.fn ? total.tp / (total.tp + total.fn) : 0;
console.log(`Total ${cases.length} examples · precision ${(precision * 100).toFixed(1)}% · recall ${(recall * 100).toFixed(1)}%`);
