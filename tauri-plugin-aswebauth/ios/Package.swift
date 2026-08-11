// swift-tools-version:5.9
// Copyright 2026 yurvon-screamo
// SPDX-License-Identifier: MIT

import PackageDescription

let package = Package(
    name: "tauri-plugin-aswebauth",
    platforms: [
        .iOS(.v13)
    ],
    products: [
        .library(
            name: "tauri-plugin-aswebauth",
            type: .static,
            targets: ["tauri-plugin-aswebauth"]
        )
    ],
    dependencies: [
        .package(name: "Tauri", path: "../.tauri/tauri-api")
    ],
    targets: [
        .target(
            name: "tauri-plugin-aswebauth",
            dependencies: [
                .byName(name: "Tauri")
            ],
            path: "Sources"
        )
    ]
)
