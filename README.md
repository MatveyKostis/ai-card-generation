# ai-card-generation

AI character card generation app (Rust + gpui). Prompt → model → character card (spec v2/v3).

## Run

```sh
cargo run                      # gpui window: prompt input, Generate button, card output
cargo run -- --generate "…"    # headless round trip: prints the card JSON
cargo test                     # card serde + extraction round-trip tests
```

## Layout

- `crates/cardgen/src/card.rs` — character card V2/V3 serde structs (spec: `chara_card_v2`/`chara_card_v3` envelope with `data`)
- `crates/cardgen/src/model.rs` — `ModelClient` trait, prompt template, JSON extraction from model output
- `crates/cardgen/src/app.rs` — gpui app shell (window, prompt input, generate action, structured card output)
- `crates/cardgen/src/generate.rs` — shared round-trip helper (prompt → model → parsed card)
- `vendor-local/xattr-src` — patched xattr 0.2.3 (maps `libc::ENOATTR` → `ENODATA`; the constant was removed from newer libc)

## Model client

No API key exists yet: `StubModelClient` (deterministic) implements `ModelClient` and
produces a valid card today. Swap in the real API client when a key lands — the UI,
card parsing, and prompt template are real and unchanged.
