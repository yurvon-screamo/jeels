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

        await page
            .getByTestId("app-loading-overlay")
            .waitFor({ state: "detached", timeout: 180_000 });
    } finally {
        await context.close();
    }
});

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
