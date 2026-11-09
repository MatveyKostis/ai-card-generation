# gpui 0.2.2 — cardgen patch

Patched copy of gpui 0.2.2 (crates.io), vendored for `[patch.crates-io]`.

## Change: DirectWrite emoji-shader failure is no longer fatal

`src/platform/windows/direct_write.rs`, marked with `TEXT-SYSTEM PATCH (cardgen)`:

1. `GPUState::new` → `new_with_fallback`: emoji vertex/pixel shader creation
   (`CreateVertexShader`/`CreatePixelShader` with the embedded
   `EMOJI_RASTERIZATION_*_BYTES` DXBC) failure now logs a warning and stores
   `None` instead of returning `Err`.
   Why: `DirectWriteComponent::new` runs during `WindowsPlatform::new`; a fatal
   shader error aborts app launch with "Error creating DirectWriteTextSystem".
   This bites when the embedded DXBC is not accepted by the target's d3d11 —
   e.g. SM6 DXIL bytecode on a DXIL-less d3d11 (Windows < 10 1703).
   Upstream's own build path (fxc, `vs_4_1`/`ps_4_1`) never hits this; our
   release build ships pre-generated shader bytes.
2. `GPUState.vertex_shader`/`pixel_shader` are now `Option`; `rasterize_color`
   returns `Ok(())` early when either is `None`, so `rasterize_glyph` falls
   back to its existing monochrome emoji path.

Everything else is unmodified upstream source.

Verified: `cargo check` passes on Linux with this patch wired (build for
`x86_64-pc-windows-gnu` validated in CI/local cross-build).
