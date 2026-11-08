//! Character card domain types.
//!
//! Single V3-shaped envelope: `chara_card_v3` is a strict superset of
//! `chara_card_v2`, so one payload round-trips as both (SillyTavern's own
//! exporter flips spec/spec_version on the same JSON). Per spec:
//! unknown fields are ignored on import but preserved on re-serialize, and
//! `extensions` content is never dropped.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

pub const SPEC: &str = "chara_card_v3";
pub const SPEC_VERSION: &str = "3.0";

/// Card envelope. `extra` captures unknown top-level fields for re-serialization.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Card {
    pub spec: String,
    pub spec_version: String,
    pub data: CardData,
    /// Unknown envelope fields, preserved verbatim (spec: never reject/destroy).
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Card {
    /// Pretty JSON string, canonical output shape.
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }

    /// Parse any accepted card shape (V2 or V3 envelope).
    pub fn from_json_str(s: &str) -> serde_json::Result<Self> {
        serde_json::from_str(s)
    }

    /// New V3 card with the canonical envelope values.
    pub fn new_v3(data: CardData) -> Self {
        Self {
            spec: SPEC.to_string(),
            spec_version: SPEC_VERSION.to_string(),
            data,
            extra: Map::new(),
        }
    }

    /// Same payload as a V2 envelope (V3 ⊇ V2, so this round-trips as both).
    pub fn to_v2_envelope(self) -> Self {
        Self {
            spec: "chara_card_v2".into(),
            spec_version: "2.0".into(),
            data: self.data,
            extra: self.extra,
        }
    }

    /// Restore the canonical V3 envelope on a parsed V2 card.
    pub fn to_v3_envelope(mut self) -> Self {
        self.spec = SPEC.into();
        self.spec_version = SPEC_VERSION.into();
        self
    }
}

/// All V2 fields required; V3-only fields optional and omitted when `None`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CardData {
    // V1 fields (required by V2 spec typing)
    pub name: String,
    pub description: String,
    pub personality: String,
    pub scenario: String,
    pub first_mes: String,
    pub mes_example: String,
    // V2 additions (required per the TS typing)
    pub creator_notes: String,
    pub system_prompt: String,
    pub post_history_instructions: String,
    pub tags: Vec<String>,
    pub creator: String,
    pub character_version: String,
    pub alternate_greetings: Vec<String>,
    /// Mandatory object, defaults to `{}`; content never dropped.
    #[serde(default = "empty_map")]
    pub extensions: Map<String, Value>,
    /// V3: required, may be empty; defaults to `[]` for bare V2 payloads.
    #[serde(default)]
    pub group_only_greetings: Vec<String>,
    // V3-only, optional, omitted when absent
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assets: Option<Vec<CardAsset>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub creator_notes_multilingual: Option<Map<String, Value>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<Vec<String>>,
    /// Unix seconds UTC; 0 = unknown.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub creation_date: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modification_date: Option<i64>,
    /// Unknown data fields, preserved verbatim (spec: ignore, don't reject).
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// V3 `assets` entry: `{type, uri, name, ext}`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CardAsset {
    #[serde(rename = "type")]
    pub kind: String,
    pub uri: String,
    pub name: String,
    pub ext: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

fn empty_map() -> Map<String, Value> {
    Map::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn full_card() -> Card {
        let data = CardData {
            name: "Aria".into(),
            description: "A wandering archivist.".into(),
            personality: "Curious, careful, dry humor.".into(),
            scenario: "A flooded archive at dusk.".into(),
            first_mes: "You made it past the waterline.".into(),
            mes_example: "<START>\n{{user}}: Hello\n{{char}}: Hi.".into(),
            creator_notes: "First card, be gentle.".into(),
            system_prompt: "You are Aria.".into(),
            post_history_instructions: "Stay in character. {{original}}".into(),
            tags: vec!["archive".into(), "fantasy".into()],
            creator: "KomaruTheCat".into(),
            character_version: "1.0".into(),
            alternate_greetings: vec!["The archive is quiet tonight.".into()],
            extensions: Map::new(),
            group_only_greetings: vec![],
            nickname: Some("Ria".into()),
            assets: Some(vec![CardAsset {
                kind: "icon".into(),
                uri: "ccdefault:".into(),
                name: "main".into(),
                ext: "png".into(),
                extra: Map::new(),
            }]),
            creator_notes_multilingual: Some(
                [(
                    "ja".to_string(),
                    serde_json::Value::String("最初のカード。".to_string()),
                )]
                .into_iter()
                .collect(),
            ),
            source: Some(vec!["https://example.com/aria".into()]),
            creation_date: Some(1_700_000_000),
            modification_date: Some(1_700_000_001),
            extra: Map::new(),
        };
        Card::new_v3(data)
    }

    fn full_card_json() -> Value {
        json!({
            "spec": "chara_card_v3",
            "spec_version": "3.0",
            "data": {
                "name": "Aria",
                "description": "A wandering archivist.",
                "personality": "Curious, careful, dry humor.",
                "scenario": "A flooded archive at dusk.",
                "first_mes": "You made it past the waterline.",
                "mes_example": "<START>\n{{user}}: Hello\n{{char}}: Hi.",
                "creator_notes": "First card, be gentle.",
                "system_prompt": "You are Aria.",
                "post_history_instructions": "Stay in character. {{original}}",
                "tags": ["archive", "fantasy"],
                "creator": "KomaruTheCat",
                "character_version": "1.0",
                "alternate_greetings": ["The archive is quiet tonight."],
                "extensions": {},
                "group_only_greetings": [],
                "nickname": "Ria",
                "assets": [
                    {"type": "icon", "uri": "ccdefault:", "name": "main", "ext": "png"}
                ],
                "creator_notes_multilingual": {"ja": "最初のカード。"},
                "source": ["https://example.com/aria"],
                "creation_date": 1_700_000_000,
                "modification_date": 1_700_000_001
            }
        })
    }

    #[test]
    fn round_trip_full_card_is_byte_identical() {
        let card = full_card();
        let json = card.to_json().unwrap();
        let parsed = Card::from_json_str(&json).unwrap();
        assert_eq!(parsed, card);
        let json2 = parsed.to_json().unwrap();
        assert_eq!(
            json, json2,
            "serialize → deserialize → serialize must be byte-identical"
        );
        // Same content as the JSON the model would emit (Value order is
        // deterministic, struct field order is declaration order).
        let as_value: Value = serde_json::from_str(&json).unwrap();
        assert_eq!(as_value, full_card_json());
    }

    #[test]
    fn v2_input_is_accepted_and_normalizes_to_v3() {
        let v2 = json!({
            "spec": "chara_card_v2",
            "spec_version": "2.0",
            "data": {
                "name": "Aria",
                "description": "A wandering archivist.",
                "personality": "p",
                "scenario": "s",
                "first_mes": "f",
                "mes_example": "m",
                "creator_notes": "cn",
                "system_prompt": "sp",
                "post_history_instructions": "phi",
                "tags": ["t"],
                "creator": "K",
                "character_version": "1.0",
                "alternate_greetings": [],
                "extensions": {"custom": 1}
            }
        });
        let card = Card::from_json_str(&serde_json::to_string(&v2).unwrap()).unwrap();
        assert_eq!(card.data.extensions.get("custom"), Some(&json!(1)));
        let v3 = card.to_v3_envelope();
        let out: Value = serde_json::from_str(&v3.to_json().unwrap()).unwrap();
        assert_eq!(out["spec"], "chara_card_v3");
        assert_eq!(out["spec_version"], "3.0");
        assert_eq!(out["data"]["group_only_greetings"], json!([]));
        assert_eq!(out["data"]["extensions"]["custom"], json!(1));
    }

    #[test]
    fn v2_envelope_payload_is_valid_v2_and_v3() {
        let card = full_card().to_v2_envelope();
        let json = card.to_json().unwrap();
        let out: Value = serde_json::from_str(&json).unwrap();
        assert_eq!(out["spec"], "chara_card_v2");
        assert_eq!(out["spec_version"], "2.0");
        assert_eq!(out["data"]["group_only_greetings"], json!([]));
        // Round-trip still byte-identical after the flip.
        let parsed = Card::from_json_str(&json).unwrap();
        assert_eq!(parsed.to_json().unwrap(), json);
    }

    #[test]
    fn unknown_fields_are_preserved() {
        let mut card = full_card();
        card.extra.insert("brand_new_top".into(), json!(42));
        card.data
            .extra
            .insert("brand_new_data".into(), json!([1, 2]));
        let json = card.to_json().unwrap();
        let parsed = Card::from_json_str(&json).unwrap();
        assert_eq!(parsed, card);
        assert_eq!(parsed.extra.get("brand_new_top"), Some(&json!(42)));
        assert_eq!(
            parsed.data.extra.get("brand_new_data"),
            Some(&json!([1, 2]))
        );
        let out: Value = serde_json::from_str(&parsed.to_json().unwrap()).unwrap();
        assert_eq!(out["brand_new_top"], json!(42));
        assert_eq!(out["data"]["brand_new_data"], json!([1, 2]));
    }

    #[test]
    fn unknown_fields_in_input_are_preserved() {
        let mut v = full_card_json();
        v["data"]["mystery"] = json!("kept");
        v["top_level_note"] = json!("also kept");
        let card = Card::from_json_str(&v.to_string()).unwrap();
        assert_eq!(card.data.extra.get("mystery"), Some(&json!("kept")));
        assert_eq!(card.extra.get("top_level_note"), Some(&json!("also kept")));
        let out: Value = serde_json::from_str(&card.to_json().unwrap()).unwrap();
        assert_eq!(out["data"]["mystery"], json!("kept"));
        assert_eq!(out["top_level_note"], json!("also kept"));
    }

    #[test]
    fn minimal_card_defaults() {
        let card = Card::new_v3(CardData {
            name: "Bare".into(),
            description: String::new(),
            personality: String::new(),
            scenario: String::new(),
            first_mes: String::new(),
            mes_example: String::new(),
            creator_notes: String::new(),
            system_prompt: String::new(),
            post_history_instructions: String::new(),
            tags: vec![],
            creator: String::new(),
            character_version: String::new(),
            alternate_greetings: vec![],
            extensions: Map::new(),
            group_only_greetings: vec![],
            nickname: None,
            assets: None,
            creator_notes_multilingual: None,
            source: None,
            creation_date: None,
            modification_date: None,
            extra: Map::new(),
        });
        let json = card.to_json().unwrap();
        let out: Value = serde_json::from_str(&json).unwrap();
        assert_eq!(out["data"]["extensions"], json!({}));
        assert_eq!(out["data"]["group_only_greetings"], json!([]));
        // V3-only fields omitted when None.
        assert!(out["data"].get("nickname").is_none());
        assert!(out["data"].get("assets").is_none());
        assert!(out["data"].get("creation_date").is_none());
    }

    #[test]
    fn missing_required_field_is_rejected() {
        let mut v = full_card_json();
        v["data"].as_object_mut().unwrap().remove("name");
        assert!(Card::from_json_str(&v.to_string()).is_err());
    }

    #[test]
    fn group_only_greetings_defaults_for_v2_payload() {
        let v2 = json!({
            "spec": "chara_card_v2",
            "spec_version": "2.0",
            "data": {
                "name": "A",
                "description": "",
                "personality": "",
                "scenario": "",
                "first_mes": "",
                "mes_example": "",
                "creator_notes": "",
                "system_prompt": "",
                "post_history_instructions": "",
                "tags": [],
                "creator": "",
                "character_version": "",
                "alternate_greetings": [],
                "extensions": {}
            }
        });
        let card = Card::from_json_str(&v2.to_string()).unwrap();
        assert_eq!(card.data.group_only_greetings, Vec::<String>::new());
    }
}
