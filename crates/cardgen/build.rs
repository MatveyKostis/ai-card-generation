fn main() {
    // gpui's build.rs gates manifest embedding behind #[cfg(target_os = "windows")],
    // which cargo evaluates for the build HOST. Cross-building from Linux silently
    // skips it, so the exe ships without a comctl32 v6 manifest and fails to load
    // with "Entry Point Not Found: TaskDialogIndirect". Embed it here instead —
    // embed-resource's build script evaluates the target env vars, so this runs
    // when cross-compiling too.
    // Matches gpui's own resources/windows/gpui.manifest.xml (DPI + Common-Controls v6).
    embed_resource::compile("resources/cardgen.rc", embed_resource::NONE)
        .manifest_required()
        .unwrap();
}
