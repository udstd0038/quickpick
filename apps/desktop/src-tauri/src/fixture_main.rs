#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // This marker is retained by the fixture handshake. A tool must verify it
    // in the PE before launch, so a mistaken production path cannot start the app.
    quickpick_lib::run_fixture("QUICKPICK_DEVTOOLS_FIXTURE_V1_6B8A5D3E");
}
