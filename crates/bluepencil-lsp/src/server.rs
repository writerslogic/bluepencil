//! The `tower_lsp::LanguageServer` implementation: tracks open documents and
//! republishes diagnostics on open/change/save.

use std::collections::HashMap;

use bluepencil_core::analysis::{adverbs, cliches, echoes, filter_words, hedges, passive};
use bluepencil_core::lexicon::WordSet;
use bluepencil_core::{Document, Format, Lexicons};
use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use crate::config::ServerConfig;
use crate::diagnostics::{echoes_to_diagnostics, findings_to_diagnostics};

struct OpenDocument {
    text: String,
    format: Format,
}

pub struct Backend {
    client: Client,
    lexicons: Lexicons,
    config: ServerConfig,
    documents: RwLock<HashMap<Url, OpenDocument>>,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            lexicons: Lexicons::default(),
            config: ServerConfig::default(),
            documents: RwLock::new(HashMap::new()),
        }
    }

    /// Picks a `Format` from the client-reported language ID, falling back to
    /// the file extension when the language ID is unrecognized (e.g. a
    /// generic "plaintext" client, or a scheme with no useful path).
    fn format_for(uri: &Url, language_id: &str) -> Format {
        match language_id {
            "markdown" => Format::Markdown,
            "fountain" => Format::Fountain,
            "plaintext" | "text" => Format::Plain,
            _ => uri.to_file_path().map(|p| Format::from_path(&p)).unwrap_or(Format::Plain),
        }
    }

    async fn analyze_and_publish(&self, uri: Url, text: String, format: Format) {
        let doc = Document::parse(uri.to_string(), text.clone(), format);

        let mut findings = Vec::new();
        findings.extend(adverbs::find(&doc, &self.lexicons));
        findings.extend(cliches::find(&doc, &self.lexicons));
        findings.extend(filter_words::find(&doc, &self.lexicons));
        findings.extend(hedges::find(&doc, &self.lexicons));
        findings.extend(passive::find(&doc, &self.lexicons));

        let mut diagnostics = findings_to_diagnostics(&doc, &findings);

        let ignore = WordSet::default();
        let echo_hits = echoes::echoes(
            &doc,
            &self.lexicons,
            self.config.echoes.window,
            self.config.echoes.min_length,
            &ignore,
            self.config.echoes.include_names,
        );
        diagnostics.extend(echoes_to_diagnostics(&doc, &uri, &echo_hits));

        self.documents.write().await.insert(uri.clone(), OpenDocument { text, format });
        self.client.publish_diagnostics(uri, diagnostics, None).await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _params: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "bluepencil-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
            capabilities: ServerCapabilities {
                // A bare `Kind(FULL)` leaves `save` unset, and clients built on
                // vscode-languageclient never register `textDocument/didSave`
                // when `save` is undefined, so `did_save` below would go dead.
                text_document_sync: Some(TextDocumentSyncCapability::Options(TextDocumentSyncOptions {
                    open_close: Some(true),
                    change: Some(TextDocumentSyncKind::FULL),
                    save: Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions { include_text: Some(false) })),
                    ..Default::default()
                })),
                ..Default::default()
            },
        })
    }

    async fn initialized(&self, _params: InitializedParams) {
        self.client.log_message(MessageType::INFO, "bluepencil-lsp initialized").await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let format = Self::format_for(&uri, &params.text_document.language_id);
        self.analyze_and_publish(uri, params.text_document.text, format).await;
    }

    async fn did_change(&self, mut params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        // Full sync: exactly one change event carrying the whole new text.
        let Some(change) = params.content_changes.pop() else { return };
        let format = self.documents.read().await.get(&uri).map(|d| d.format).unwrap_or(Format::Plain);
        self.analyze_and_publish(uri, change.text, format).await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let uri = params.text_document.uri;
        let format = self.documents.read().await.get(&uri).map(|d| d.format).unwrap_or(Format::Plain);
        let text = match params.text {
            Some(text) => text,
            None => match self.documents.read().await.get(&uri) {
                Some(doc) => doc.text.clone(),
                None => return,
            },
        };
        self.analyze_and_publish(uri, text, format).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.documents.write().await.remove(&params.text_document.uri);
        self.client.publish_diagnostics(params.text_document.uri, Vec::new(), None).await;
    }
}
