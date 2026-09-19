//! Server-side analysis configuration.
//!
//! TODO: read a `bluepencil.toml` near the open file (same schema as the CLI's
//! `crates/bluepencil/src/config.rs`) so per-project echo/lexicon overrides
//! apply here too. Deferred for v1: locating the right config relative to an
//! LSP `Url` (workspace root vs. file path, multi-root workspaces) is a larger
//! lift than a single-binary CLI invocation, so v1 ships fixed defaults.

#[derive(Debug, Clone, Copy)]
pub struct EchoesConfig {
    pub window: usize,
    pub min_length: usize,
    pub include_names: bool,
}

impl Default for EchoesConfig {
    fn default() -> Self {
        // Matches the CLI's own default (`crates/bluepencil/src/config.rs`'s
        // `impl Default for Echoes`) so a file flags the same echoes here and
        // via `bluepencil echoes` with no `bluepencil.toml` present.
        Self { window: 50, min_length: 4, include_names: false }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ServerConfig {
    pub echoes: EchoesConfig,
}
