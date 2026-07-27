import { test as base, createBdd } from "playwright-bdd";
import { withAuthenticatedPage } from "../helpers/auth";

/**
 * BDD test base with authenticated page fixture.
 *
 * Extends playwright-bdd's base (NOT @playwright/test) so that createBdd()
 * can bind step definitions. The `page` fixture is overridden to provide
 * an authenticated page — same pattern as testWithFreshUser in
 * fixtures/onboarding.fixture.ts.
 *
 * Step definitions import { Given, When, Then } from this file.
 */
export const test = base.extend<object, object>({
    page: [
        async ({ browser }, use) => {
            await withAuthenticatedPage(browser, use);
        },
        // uiLogin can retry up to 3 times, each with a 60s waitForURL timeout,
        // plus WASM cold load. Default fixture timeout (60s) is too tight.
        { timeout: 180_000 },
    ],
});

export const { Given, When, Then } = createBdd(test);
