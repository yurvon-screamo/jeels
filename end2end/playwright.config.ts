import { defineConfig, devices } from "@playwright/test";
import { defineBddConfig } from "playwright-bdd";

const isCI = !!process.env.CI;

const bddTestDir = defineBddConfig({
    features: "./bdd/features",
    steps: "./bdd/**/*.ts",
});

export default defineConfig({
    testDir: "./tests",
    timeout: 60000,
    expect: {
        timeout: 10000,
    },
    fullyParallel: true,
    forbidOnly: isCI,
    retries: isCI ? 2 : 0,
    workers: isCI ? 2 : 2,
    reporter: isCI
        ? [["blob", { outputDir: "blob-report" }]]
        : [["html", { open: "on-failure", host: "0.0.0.0" }]],
    use: {
        baseURL: "http://localhost:1420",
        trace: "on-first-retry",
        screenshot: "only-on-failure",
        video: "retain-on-failure",
        // Constrain the renderer memory budget so that desktop CI fails like
        // an iOS WKWebView instead of silently surviving memory-hungry pages
        // (jetsam kills the iOS app at ~1.5 GB; the black-screen set-import
        // crash was invisible on desktop for exactly this reason).
        // 1 GB c-group-style V8 heap cap: JS heap + WASM linear memory grow
        // inside this budget, so a page that would OOM a phone OOMs the test.
        launchOptions: {
            args: ["--js-flags=--max-old-space-size=1024", "--memory-pressure-off"],
        },
    },
    projects: [
        {
            name: "chromium",
            use: {
                ...devices["Desktop Chrome"],
            },
        },
        {
            name: "bdd",
            testDir: bddTestDir,
            // BDD scenarios span multiple pages and frequently reload WASM
            // (onboarding, lesson lifecycle) or do a full login twice (the
            // sync roundtrip: fixture login → logout → re-login). Each UI
            // login is a cold WASM load + TrailBase auth round-trip, so the
            // default 60s test timeout is too tight and aborts mid-flow.
            // 180s matches the `page` fixture timeout in bdd/fixtures.ts.
            timeout: 180000,
            use: {
                ...devices["Desktop Chrome"],
            },
        },
    ],
    webServer: [
        {
            command: "npx serve ../cdn -p 8080 --no-clipboard --cors",
            port: 8080,
            reuseExistingServer: !isCI,
            timeout: 30000,
            stdout: "pipe",
            stderr: "pipe",
        },
        {
            command: isCI
                ? "npx serve ../origa_ui/dist -s -p 1420 --no-clipboard"
                : "cd ../origa_ui && trunk serve",
            port: 1420,
            reuseExistingServer: !isCI,
            timeout: 600000,
            stdout: "pipe",
            stderr: "pipe",
            env: {
                ORIGA_CDN_BASE_URL: "http://localhost:8080",
                TRAILBASE_URL: "http://127.0.0.1:4000",
            },
        },
    ],
    globalSetup: "./global-setup.ts",
    globalTeardown: "./global-teardown.ts",
});
