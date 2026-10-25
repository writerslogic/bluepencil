#!/usr/bin/env bash
set -euo pipefail

site_dir="${1:-docs/book}"
mkdir -p "$site_dir"
site_root="$(cd "$site_dir" && pwd)"

mdbook build docs --dest-dir "$site_root/docs"
cp -R docs/playground/. "$site_root/"
mkdir -p "$site_root/playground"
cp docs/redirects/playground/index.html "$site_root/playground/index.html"
wasm-pack build crates/bluepencil-web --release --target web --out-dir "$site_root/pkg"
node docs/redirects/build.mjs "$site_root"

test -f "$site_root/index.html"
test -f "$site_root/docs/index.html"
test -f "$site_root/pkg/bluepencil_web.js"
test -f "$site_root/editors.html"
