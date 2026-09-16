//! Text analysis engine behind the `bluepencil` CLI.

pub mod analysis;
pub mod document;
pub mod lexicon;
pub mod location;
pub mod parse;
pub mod span;
pub mod stats;
pub mod text;

pub use analysis::Finding;
pub use document::{Document, Format, Heading, Paragraph, Sentence, Word};
pub use lexicon::Lexicons;
pub use location::Position;
pub use span::Span;
