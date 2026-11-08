use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Card format version: v2 (`spec: "chara_card_v2"`) or v3 (`spec: "chara_card_v3"`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CardSpec {
    CharaCardV2,
    CharaCardV3,
}

/// Optional V3 asset entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AssetEntry {
    pub r#type: String,
    pub uri: String,
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ext: Option<String>,
}

/// Optional V3 embedded lorebook.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CharacterBook {
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub entries: Vec<BookEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BookEntry {
    pub keys: Vec<String>,
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Character card data. All model-facing fields follow the spec; unknown keys
/// survive round-trips through `extensions`-adjacent `extra`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CharacterCard {
    pub name: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub personality: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub scenario: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub first_mes: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub mes_example: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub creator_notes: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub post_history_instructions: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub alternate_greetings: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub character_book: Option<CharacterBook>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub creator: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub character_version: Option<String>,
    #[serde(default, skip_serializing_if = "Value::is_null")]
    pub extensions: Value,
    /// V3 only.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub assets: Vec<AssetEntry>,
    /// V3 only.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub group_only_greetings: Vec<String>,
}

/// A serialized card file: `{"spec": "chara_card_v2"|"chara_card_v3", "data": {...}}`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CardFile {
    pub spec: CardSpec,
    pub data: CharacterCard,
}

impl CardFile {
    pub fn v2(data: CharacterCard) -> Self {
        Self { spec: CardSpec::CharaCardV2, data }
    }

    /// V3 envelope constructor; used by Backend Builder's card layer.
    #[allow(dead_code)]
    pub fn v3(data: CharacterCard) -> Self {
        Self { spec: CardSpec::CharaCardV3, data }
    }

    /// Convenience constructor: card with only the required v2 fields filled.
    /// Used by Backend Builder's template layer; not wired into the UI yet.
    #[allow(dead_code)]
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self::v2(CharacterCard {
            name: name.into(),
            description: description.into(),
            personality: String::new(),
            scenario: String::new(),
            first_mes: String::new(),
            mes_example: String::new(),
            creator_notes: None,
            system_prompt: None,
            post_history_instructions: None,
            alternate_greetings: Vec::new(),
            character_book: None,
            tags: Vec::new(),
            creator: None,
            character_version: None,
            extensions: serde_json::json!({}),
            assets: Vec::new(),
            group_only_greetings: Vec::new(),
        })
    }

    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Parses a card file; used by Backend Builder's JSON-extraction layer.
    #[allow(dead_code)]
    pub fn from_json(json: &str) -> Result<Self> {
        Ok(serde_json::from_str(json)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_v2_card() {
        let card = CardFile::new("Aria", "A forest ranger with a secrets-laden past.");
        let json = card.to_json().unwrap();
        let parsed = CardFile::from_json(&json).unwrap();
        assert_eq!(parsed, card);
        assert_eq!(json, CardFile::from_json(&json).unwrap().to_json().unwrap());
    }

    #[test]
    fn parses_v3_card_with_assets() {
        let json = r#"{
            "spec": "chara_card_v3",
            "data": {
                "name": "Kit",
                "description": "A tinkerer.",
                "assets": [{ "type": "icon", "uri": "file://kit.png", "name": "icon" }],
                "group_only_greetings": ["Hi all!"],
                "extensions": { "vendor": { "x": 1 } }
            }
        }"#;
        let card = CardFile::from_json(json).unwrap();
        assert_eq!(card.spec, CardSpec::CharaCardV3);
        assert_eq!(card.data.assets.len(), 1);
        assert_eq!(card.data.group_only_greetings.len(), 1);
        let round = CardFile::from_json(&card.to_json().unwrap()).unwrap();
        assert_eq!(round, card);
    }
}
