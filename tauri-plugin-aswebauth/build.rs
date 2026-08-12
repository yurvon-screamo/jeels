// Copyright 2026 yurvon-screamo
// SPDX-License-Identifier: MIT

const COMMANDS: &[&str] = &["start_auth"];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).ios_path("ios").build();
}
