mod manuscript;

use manuscript::{BROWSER_INPUT_BYTES, BrowserManuscript, parse_format};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Manuscript {
    inner: BrowserManuscript,
}

#[wasm_bindgen]
impl Manuscript {
    #[wasm_bindgen(constructor)]
    pub fn new(source: &str, format: &str) -> Result<Manuscript, JsValue> {
        let format = parse_format(format).map_err(js_error)?;
        let inner = BrowserManuscript::new(source, format, BROWSER_INPUT_BYTES).map_err(js_error)?;
        Ok(Self { inner })
    }

    pub fn stages(&self) -> usize {
        BrowserManuscript::STAGES.len()
    }

    pub fn next_stage(&self) -> String {
        self.inner.next_stage().to_string()
    }

    pub fn advance(&mut self) {
        self.inner.advance();
    }

    pub fn summary(&self) -> Result<String, JsValue> {
        serialize(&self.inner.summary().map_err(js_error)?)
    }

    pub fn visible(&self, from: usize, to: usize, enabled: &str) -> Result<String, JsValue> {
        serialize(&self.inner.visible(from, to, enabled).map_err(js_error)?)
    }

    pub fn chart(&self, from: usize, to: usize, interval: usize) -> Result<String, JsValue> {
        serialize(&self.inner.chart(from, to, interval).map_err(js_error)?)
    }
}

#[wasm_bindgen]
pub fn analyze(source: &str, format: &str) -> Result<String, JsValue> {
    analyze_with_limit(source, format, BROWSER_INPUT_BYTES)
}

#[wasm_bindgen]
pub fn analyze_full(source: &str, format: &str) -> Result<String, JsValue> {
    analyze_with_limit(source, format, 1024 * 1024)
}

fn analyze_with_limit(source: &str, format: &str, max_bytes: usize) -> Result<String, JsValue> {
    let format = parse_format(format).map_err(js_error)?;
    let mut manuscript = BrowserManuscript::new(source, format, max_bytes).map_err(js_error)?;
    while !manuscript.is_finished() {
        manuscript.advance();
    }
    serialize(&manuscript.legacy_analysis())
}

fn serialize(value: &impl serde::Serialize) -> Result<String, JsValue> {
    serde_json::to_string(value).map_err(|_| js_error("Could not prepare the analysis results."))
}

fn js_error(message: impl AsRef<str>) -> JsValue {
    JsValue::from_str(message.as_ref())
}
