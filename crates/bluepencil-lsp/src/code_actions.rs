//! `textDocument/codeAction` quick fixes (e.g. "remove this word" for a
//! flagged adverb or filter word).
//!
//! TODO: deferred for v1. Diagnostics publishing (`server.rs`, `diagnostics.rs`)
//! covers the core value of the server; quick fixes need per-rule logic to
//! decide which findings are safely removable without breaking the sentence
//! (a flagged cliché or passive-voice span usually isn't a single deletable
//! word, only some adverb/filter-word/hedge findings are), which is scoped as
//! follow-up work rather than v1.
