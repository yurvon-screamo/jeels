/**
 * Diagnostic for the N1-stress sync scenario: seeds the full-corpus user
 * on device 1 (the exact seed the BDD scenario uses), then logs a second,
 * fresh browser context (empty IndexedDB partition) into the same account
 * and reports where the app settles plus the remote record state before
 * and after — distinguishing "device 1 never persisted the
 * onboarding-completed sentinel" from "device 2 lost it".
 *
 * Run: npx -y tsx scripts/diag-n1-sentinel.ts (from end2end/) with the
 * usual stack up (trailbase 4000 / cdn 8080 / app 1420).
 */
import { chromium } from "@playwright/test";

import { setupTestUser, uiLogin } from "../helpers/auth";
import { completeN1StressSeed } from "../helpers/onboarding";

const APP_BASE = "http://localhost:1420";

interface RemoteRecord {
    email: string;
    importedSets: string | null;
    hasSentinel: boolean;
    setsPreview: string[];
}

async function fetchRemoteRecord(email: string): Promise<RemoteRecord> {
    const { getTrailBaseUrl } = await import("../config");
    const { getAdminToken } = await import("../fixtures/admin");
    const base = getTrailBaseUrl();
    const auth = await getAdminToken();

    const listResponse = await fetch(
        `${base}/api/records/v1/domain_user?filter[email][$eq]=${encodeURIComponent(email)}`,
        { headers: { Authorization: `Bearer ${auth.token}` } },
    );
    if (!listResponse.ok) {
        throw new Error(`admin list failed: ${listResponse.status}`);
    }
    const payload = (await listResponse.json()) as { records: Array<{ imported_sets?: string }> };
    const record = payload.records[0];
    const importedSets = record?.imported_sets ?? null;
    const parsed = importedSets ? (JSON.parse(importedSets) as string[]) : [];
    return {
        email,
        importedSets,
        hasSentinel: parsed.includes("__onboarding_completed__"),
        setsPreview: parsed.slice(0, 10),
    };
}

/**
 * Neutralizes the trunk live-reload script (baked into the served
 * index.html): on any WS reconnect it force-calls `window.location.reload()`,
 * which aborts the WASM load, interrupts Playwright gotos and restarts the
 * restore mid-flight — pure dev-tooling noise masking the behaviour under
 * test. Blocking just the reload method keeps everything else untouched.
 */
async function blockTrunkReload(context: import("@playwright/test").BrowserContext): Promise<void> {
    await context.addInitScript(() => {
        try {
            Object.defineProperty(window.Location.prototype, "reload", {
                configurable: true,
                value: () => {
                    console.log("[diag] trunk live-reload blocked");
                },
            });
        } catch {
            /* prototype not patchable — proceed without the guard */
        }
    });
}

async function main(): Promise<void> {
    const userCtx = await setupTestUser();
    console.log(`[diag] test user: ${userCtx.email}`);

    const browser = await chromium.launch();
    const context = await browser.newContext({ baseURL: APP_BASE, locale: "ru-RU" });
    const page = await context.newPage();
    await page.setViewportSize({ width: 1280, height: 720 });
    // Device-1 listeners: login flakes must leave a trace (url, console).
    page.on("pageerror", (err) => console.log(`[dev1 pageerror] ${String(err).slice(0, 300)}`));
    page.on("framenavigated", (frame) => {
        if (frame === page.mainFrame()) console.log(`[dev1 navigated] ${frame.url()}`);
    });
    await context.addInitScript(() => {
        if (window.location.origin === "http://localhost:1420") {
            window.localStorage.setItem("origa_resource_download_consented", "true");
        }
    });
    // NEUTRALIZE the trunk live-reload script (baked into the served
    // index.html): on any WS reconnect it force-reloads the page, which
    // aborts the WASM load, interrupts Playwright gotos and restarts the
    // restore mid-flight — pure dev-tooling noise masking the behaviour
    // under test. Strip the script from the HTML via a route interceptor.
    await blockTrunkReload(context);

    await uiLogin(page, userCtx.email, userCtx.password);
    await completeN1StressSeed(page);
    console.log("[diag] seed complete, waiting 5s for trailing writes…");
    await page.waitForTimeout(5_000);
    const before = await fetchRemoteRecord(userCtx.email);
    console.log("[diag] remote AFTER seed:", JSON.stringify(before, null, 2));

    // Free the heavy first-device wasm instance before booting the second:
    // two debug-build apps in one Chrome starve each other.
    await page.close();
    await context.close();

    // Device 2: fresh IndexedDB partition, same account.
    const context2 = await browser.newContext({ baseURL: APP_BASE, locale: "ru-RU" });
    const page2 = await context2.newPage();
    await page2.setViewportSize({ width: 1280, height: 720 });
    // Diagnostic listeners: the goto interruption cause must be visible.
    page2.on("console", (msg) => {
        if (msg.type() === "error" || msg.type() === "warning") {
            console.log(`[dev2 console.${msg.type()}] ${msg.text().slice(0, 300)}`);
        }
    });
    page2.on("pageerror", (err) => console.log(`[dev2 pageerror] ${String(err).slice(0, 300)}`));
    page2.on("framenavigated", (frame) => {
        if (frame === page2.mainFrame()) console.log(`[dev2 navigated] ${frame.url()}`);
    });
    await context2.addInitScript(() => {
        if (window.location.origin === "http://localhost:1420") {
            window.localStorage.setItem("origa_resource_download_consented", "true");
        }
    });
    // Neutralize the trunk live-reload script — see device 1 comment.
    await blockTrunkReload(context2);

    // uiLogin already retries internally, but its plain goto can race the
    // app's own redirects on this slow-restore device ("navigation
    // interrupted") — wrap it in an outer retry with a reload between
    // attempts.
    let loggedIn = false;
    for (let attempt = 1; attempt <= 3 && !loggedIn; attempt++) {
        try {
            await uiLogin(page2, userCtx.email, userCtx.password);
            loggedIn = true;
        } catch (e) {
            console.log(
                `[diag] device2 uiLogin attempt ${attempt} failed: ${String(e).slice(0, 600)}`,
            );
            await page2.reload({ waitUntil: "domcontentloaded" }).catch(() => {});
        }
    }
    if (!loggedIn) {
        throw new Error("device2 login never succeeded");
    }
    console.log("[diag] device2 login submitted, settling…");

    await page2.waitForTimeout(240_000);
    console.log(`[diag] device2 url after settle: ${page2.url()}`);
    const homeVisible = await page2
        .getByTestId("home-page")
        .isVisible()
        .catch(() => false);
    console.log(`[diag] device2 home-page visible: ${homeVisible}`);

    const after = await fetchRemoteRecord(userCtx.email);
    console.log("[diag] remote AFTER device2:", JSON.stringify(after, null, 2));

    await browser.close();
    await userCtx.cleanup();
}

main().catch((error) => {
    console.error("[diag] failed:", error);
    process.exit(1);
});
