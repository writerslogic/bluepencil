# Releasing bluepencil

The CLI, core library, language server, and VS Code extension use the same release version. The npm package `bluepencil-node` is released separately by `release-node.yml`.

1. Update the workspace version and `bluepencil-core` path dependency in `Cargo.toml`, the VS Code version in `editors/vscode/package.json` and its lockfile, and `CHANGELOG.md`.
2. Run the checks used by CI: `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --all-targets`, `cargo +1.88 check --workspace`, `cargo deny check`, `bash scripts/build-pages.sh`, and `npx @vscode/vsce package` from `editors/vscode` after `npm ci`. Wait for CI and Audit to pass on `main`.
3. Check `dist plan --tag=vX.Y.Z`, then create and push the tag with `git tag -a vX.Y.Z -m 'bluepencil vX.Y.Z'` and `git push origin vX.Y.Z`. The release workflow builds binaries and installer scripts for macOS, Linux, and Windows and attaches generated Homebrew formulae to the GitHub release.
4. The release workflow pushes `bluepencil.rb` and `bluepencil-lsp.rb` to `writerslogic/homebrew-tap` when `HOMEBREW_TAP_TOKEN` has write access there. Without that secret, copy both formula assets from the GitHub release into the tap and push them manually. Verify `brew info writerslogic/tap/bluepencil` and `brew info writerslogic/tap/bluepencil-lsp` show the new version.
5. Publish crates.io packages in dependency order: `cargo publish -p bluepencil-core`, then `cargo publish -p bluepencil` and `cargo publish -p bluepencil-lsp` once the core version is available in the registry. `bluepencil-node` and `bluepencil-web` are not published as Rust crates.
6. Attach `editors/vscode/bluepencil-X.Y.Z.vsix` to the GitHub release. To publish the extension in the VS Code Marketplace, add a `VSCE_PAT` repository secret with Marketplace manage permission for the `writerslogic` publisher, then run `publish-vscode.yml` against the release tag. The VSIX remains installable from the GitHub release independently of the Marketplace.

Verify the public writing desk at `https://writerslogic.github.io/bluepencil/` and the guide at `https://writerslogic.github.io/bluepencil/docs/` after the Docs workflow finishes.
