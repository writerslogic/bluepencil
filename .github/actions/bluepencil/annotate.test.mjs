import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { chmodSync, mkdtempSync, mkdirSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { delimiter, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const script = fileURLToPath(new URL("./annotate.mjs", import.meta.url));

test("annotates a tracked passage and escapes workflow command text", () => {
  const dir = mkdtempSync(join(tmpdir(), "bluepencil-action-test-"));
  try {
    execFileSync("git", ["init", "-q", dir]);
    writeFileSync(join(dir, "scene.md"), "The gravel was cold. The gravel was wet.\n");
    execFileSync("git", ["-C", dir, "add", "scene.md"]);
    const bin = join(dir, "bin");
    mkdirSync(bin);
    const mock = join(bin, "bluepencil");
    writeFileSync(mock, "#!/usr/bin/env node\nconsole.log(JSON.stringify({revisions:[{file:'scene.md',line:1,column:26,rule:'echo',message:'word repeats\\n::error::not an error'}]}));\n");
    chmodSync(mock, 0o755);
    const output = execFileSync(process.execPath, [script], {
      cwd: dir,
      env: { ...process.env, PATH: `${bin}${delimiter}${process.env.PATH}`, BLUEPENCIL_LIMIT: "5", BLUEPENCIL_BASE_SHA: "" },
      encoding: "utf8",
    });
    assert.match(output, /::notice file=scene\.md,line=1,col=26,title=bluepencil%3A echo::word repeats%0A::error::not an error/);
    assert.match(output, /reviewed 1 file\(s\) and found 1 observation\(s\)/);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});
