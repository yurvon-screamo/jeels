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
export const test = base.extend<object>({
    page: async ({ browser }, use) => {
        await withAuthenticatedPage(browser, use);
    },
});

export const { Given, When, Then } = createBdd(test);
