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
