use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::card::CardFile;

/// Prompt sent to the model; `prompt` is the user's character description.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateRequest {
    pub prompt: String,
}

/// The model's raw reply (text expected to embed a card JSON object).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateResponse {
    pub raw: String,
}

#[async_trait]
pub trait ModelClient: Send + Sync {
    /// Runs one prompt → raw model text round trip.
    async fn generate(&self, request: &GenerateRequest) -> Result<GenerateResponse>;
}

/// Builds the system + user prompt pair sent to the model. The model is told to
/// reply with a single character-card JSON object (spec v2/v3 envelope).
pub fn build_prompt(prompt: &str) -> String {
    format!(
        "You are a character card generator. Reply with EXACTLY ONE JSON object, \
no other text, using this shape:\n\
{{\n  \"spec\": \"chara_card_v2\" | \"chara_card_v3\",\n  \"data\": {{\n    \
\"name\": string,\n    \"description\": string,\n    \"personality\": string,\n    \
\"scenario\": string,\n    \"first_mes\": string,\n    \"mes_example\": string,\n    \
\"creator_notes\": string (optional),\n    \"system_prompt\": string (optional),\n    \
\"post_history_instructions\": string (optional),\n    \"alternate_greetings\": [string],\n    \
\"character_book\": {{...}} (optional),\n    \"tags\": [string],\n    \"creator\": string (optional),\n    \
\"character_version\": string (optional),\n    \"extensions\": object,\n    \
\"assets\": [{{\"type\": string, \"uri\": string, \"name\": string}}] (v3 only),\n    \
\"group_only_greetings\": [string] (v3 only)\n  }}\n}}\n\n\
Character request: {prompt}"
    )
}

/// Deterministic stub client: no API key exists yet, so this synthesizes a
/// valid card from the prompt. Swap for the real API client when a key lands.
pub struct StubModelClient;

#[async_trait]
impl ModelClient for StubModelClient {
    async fn generate(&self, request: &GenerateRequest) -> Result<GenerateResponse> {
        let name = request
            .prompt
            .split_whitespace()
            .take(4)
            .collect::<Vec<_>>()
            .join(" ");
        let name = if name.is_empty() { "Unnamed".to_string() } else { name };
        let card = CardFile::v2(serde_json::from_value(serde_json::json!({
            "name": name,
            "description": format!("A character born from the request: {}", request.prompt),
            "personality": "Curious, resourceful, a little chaotic.",
            "scenario": format!("{} steps out of the prompt and into the scene.", name),
            "first_mes": format!("*{} looks up.* So — you're the one who summoned me?", name),
            "mes_example": "<START>\n{{user}}: Hello\n{{char}}: Hello back.",
            "tags": ["stub", "generated"],
            "creator": "cardgen stub",
            "character_version": "0.1.0",
            "extensions": {}
        }))?);
        Ok(GenerateResponse { raw: card.to_json()? })
    }
}

/// Extracts a card JSON object from raw model text: direct parse first, then
/// fenced ```json blocks, then the first balanced `{ ... }` span.
pub fn extract_card_json(raw: &str) -> Result<CardFile> {
    if let Ok(card) = serde_json::from_str::<CardFile>(raw.trim()) {
        return Ok(card);
    }
    for chunk in raw.split("```") {
        let chunk = chunk.trim();
        let chunk = chunk.strip_prefix("json").unwrap_or(chunk).trim();
        if chunk.starts_with('{')
            && let Ok(card) = serde_json::from_str::<CardFile>(chunk)
        {
            return Ok(card);
        }
    }
    let bytes = raw.as_bytes();
    let mut start = None;
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'"' if !escaped => in_string = !in_string,
            b'\\' if in_string => escaped = !escaped,
            b'{' if !in_string => {
                if depth == 0 {
                    start = Some(i);
                }
                depth += 1;
            }
            b'}' if !in_string => {
                depth = depth.saturating_sub(1);
                if depth == 0
                    && let Some(s) = start
                    && let Ok(card) = serde_json::from_str::<CardFile>(&raw[s..=i])
                {
                    return Ok(card);
                }
            }
            _ => {}
        }
        if b != b'\\' {
            escaped = false;
        }
    }
    anyhow::bail!("no character card JSON found in model output")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::CardSpec;

    #[test]
    fn stub_produces_parseable_card() {
        let client = StubModelClient;
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let response = runtime
            .block_on(client.generate(&GenerateRequest { prompt: "a grumpy wizard".into() }))
            .unwrap();
        let card = extract_card_json(&response.raw).unwrap();
        assert_eq!(card.spec, CardSpec::CharaCardV2);
        assert!(card.data.name.contains("wizard"));
    }

    #[test]
    fn extracts_from_fenced_block() {
        let raw = "Sure! Here you go:\n```json\n{\"spec\":\"chara_card_v2\",\"data\":{\"name\":\"X\",\"description\":\"Y\"}}\n```\nEnjoy.";
        let card = extract_card_json(raw).unwrap();
        assert_eq!(card.data.name, "X");
    }

    #[test]
    fn extracts_balanced_object_from_prose() {
        let raw = "prefix {\"spec\":\"chara_card_v3\",\"data\":{\"name\":\"Z\",\"description\":\"W\"}} suffix";
        let card = extract_card_json(raw).unwrap();
        assert_eq!(card.data.name, "Z");
    }
}
