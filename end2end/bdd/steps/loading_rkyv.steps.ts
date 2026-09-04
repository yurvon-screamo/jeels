import { expect } from "@playwright/test";
import { uiLogin } from "../../helpers/auth";
import { Given, Then } from "../fixtures";

/**
 * Clean-cache fast-path scenario: a fresh browser context (empty Cache API)
 * logs in and fully loads the app while every network request URL is
 * recorded. The fast path must fetch the two rkyv blobs and must NOT touch
 * the fallback sources (vocabulary JSON chunks, furigana text) — a silent
 * fallback regression fails here instead of green-lighting CI.
 *
 * Scope note: the offline-bundle flow legitimately downloads chunks/txt, so
 * this assertion only covers the startup scenario driven below; other
 * scenarios must not reuse the request log.
 */
Given('приложение загрузилось с чистым кэшем', async ({ browser, testUser, cdnRequestLog }) => {
    const context = await browser.newContext();
    const page = await context.newPage();

    try {
        page.on("request", (request) => {
            cdnRequestLog.push(request.url());
        });

        await uiLogin(page, testUser.email, testUser.password);

        // uiLogin's race may return as soon as the login form unmounts —
        // the login flow (and thus the dictionary bootstrap behind
        // ProtectedRoute) can still be in flight. Wait for the terminal
        // navigation first (fresh user → /onboarding, restored → /home),
        // then for the loading overlay to run its full course: this is
        // what actually issues the rkyv requests this scenario asserts on.
        await page
            .waitForURL(/\/(home|onboarding)/, { timeout: 90_000 })
            .catch(() => tracing_note_stuck_url(page));
        await page
            .getByTestId("app-loading-overlay")
            .waitFor({ state: "detached", timeout: 180_000 });
    } finally {
        await context.close();
    }
});

/** Best-effort diagnostics when the terminal navigation never happened. */
function tracing_note_stuck_url(page: import("@playwright/test").Page): void {
    console.warn(`[loading_rkyv] terminal navigation timeout; url=${page.url()}`);
}

Then('словари загружены через rkyv-блобы', async ({ cdnRequestLog }) => {
    const fallbackRequests = cdnRequestLog.filter(
        (url) =>
            url.includes("dictionary/chunk_") || url.includes("JmdictFurigana.txt"),
    );
    expect(
        fallbackRequests,
        `fallback sources must not be fetched on the fast path, got: ${fallbackRequests.join(", ")}`,
    ).toHaveLength(0);

    const rkyvRequests = cdnRequestLog.filter((url) => url.endsWith(".rkyv"));
    expect(rkyvRequests.length, "both rkyv blobs must be fetched").toBeGreaterThanOrEqual(2);
});
