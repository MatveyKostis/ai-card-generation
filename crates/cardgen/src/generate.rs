use card_domain::Card;
use crate::model::{
    GenerateRequest, ModelClient, StubModelClient, build_prompt, extract_card_json,
};

/// Shared round-trip helper: prompt → model → parsed card. Used by both the
/// headless CLI path and the UI generate action. The prompt is built with
/// `build_prompt` and carried in the request; `v3` selects the canonical V3
/// envelope, default is V2 (import-compat floor for ST-style consumers).
pub fn generate_and_print(prompt: &str, v3: bool) -> anyhow::Result<String> {
    let mut request = GenerateRequest {
        prompt: build_prompt(prompt),
    };
    let client = StubModelClient;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let response = runtime.block_on(client.generate(&request))?;
    let card: Card = extract_card_json(&response.raw)?;
    if v3 {
        // The stub already emits V3; a real V2 reply is normalized back to V3.
        request.prompt = prompt.to_string();
        return Ok(card.to_json()?);
    }
    Ok(card.to_v2_envelope().to_json()?)
}
