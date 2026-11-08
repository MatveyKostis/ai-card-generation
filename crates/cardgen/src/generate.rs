use crate::card::CardFile;
use crate::model::{GenerateRequest, ModelClient, StubModelClient, build_prompt, extract_card_json};

/// Shared round-trip helper: prompt → model → parsed card. Used by both the
/// headless CLI path and the UI generate action. Blocking is acceptable here
/// because callers already run outside the UI render path.
pub fn generate_and_print(prompt: &str) -> Result<String, anyhow::Error> {
    let request = GenerateRequest { prompt: prompt.to_string() };
    let _built = build_prompt(&request.prompt);
    let client = StubModelClient;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let response = runtime.block_on(client.generate(&request))?;
    let card: CardFile = extract_card_json(&response.raw)?;
    card.to_json()
}
