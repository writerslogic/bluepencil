#!/usr/bin/env bash
set -euo pipefail

site_dir="${1:-docs/book}"
mkdir -p "$site_dir"
site_root="$(cd "$site_dir" && pwd)"

mdbook build docs --dest-dir "$site_root/docs"
cp docs/playground/*.html docs/playground/*.css docs/playground/*.js docs/playground/*.png "$site_root/"
mkdir -p "$site_root/fonts"
cp -R docs/playground/fonts/. "$site_root/fonts/"
mkdir -p "$site_root/playground"
cp docs/redirects/playground/index.html "$site_root/playground/index.html"
wasm-pack build crates/bluepencil-web --release --target web --out-dir "$site_root/pkg"
npm --prefix docs/playground run build -- "$site_root"
node docs/redirects/build.mjs "$site_root"

test -f "$site_root/index.html"
test -f "$site_root/docs/index.html"
test -f "$site_root/pkg/bluepencil_web.js"
test -f "$site_root/analysis-worker.js"
test -f "$site_root/playground.js.LICENSE.txt"
test ! -d "$site_root/node_modules"
test -f "$site_root/editors.html"
