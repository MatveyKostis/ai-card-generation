# ai-card-generation

AI character card generation app (Rust + gpui). Prompt → model → character card (spec v2/v3).

## Run

```sh
cargo run                                    # gpui window: prompt input, Generate button, card output
cargo run -- --generate "…"                  # headless round trip: prints the card JSON (V2 envelope)
cargo run -- --generate "…" --spec=v3        # headless round trip with the canonical V3 envelope
cargo test                                   # card serde + extraction round-trip tests
```

## Layout

- `crates/card-domain/` — character card serde crate: one V3-shaped envelope (`Card`) that round-trips as both V2 and V3, unknown-field preservation via `serde(flatten)`
- `crates/cardgen/src/model.rs` — `ModelClient` trait, prompt template, JSON extraction from model output
- `crates/cardgen/src/app.rs` — gpui app shell (window, prompt input, async generate action, structured card output)
- `crates/cardgen/src/generate.rs` — shared round-trip helper (prompt → model → parsed card)
- `vendor-local/xattr-src` — patched xattr 0.2.3 (maps `libc::ENOATTR` → `ENODATA`; the constant was removed from newer libc)

## Model client

No API key exists yet: `StubModelClient` (deterministic) implements `ModelClient` and
produces a valid card today. Swap in the real API client when a key lands — the UI,
card parsing, and prompt template are real and unchanged.
