import { test as base, createBdd } from "playwright-bdd";
import { setupTestUser, uiLogin, type TestUserContext } from "../helpers/auth";

/**
 * BDD test base with an authenticated page fixture + the matching test account.
 *
 * Extends playwright-bdd's base (NOT @playwright/test) so that createBdd()
 * can bind step definitions. Two fixtures are provided:
 *
 * - `testUser` — creates a fresh test account once per test (setupTestUser)
 *   and exposes its email/password so steps can re-login into the SAME
 *   identity after a logout. Cleanup runs on teardown.
 * - `page` — opens a browser context and logs into `testUser`. Equivalent to
 *   the old `withAuthenticatedPage` behaviour, but reuses the shared account
 *   so a "logout → login again" roundtrip targets the same user.
 *
 * Most steps destructure only `{ page }`; re-login flows add `testUser`.
 *
 * uiLogin can retry up to 3 times, each with a 60s waitForURL timeout, plus
 * WASM cold load. The default fixture timeout (60s) is too tight, so the page
 * fixture overrides it.
 *
 * Step definitions import { Given, When, Then } from this file.
 */
export const test = base.extend<
    {
        testUser: TestUserContext;
        acqTrainingLog: string[];
        acqKnownCard: { value: string | null };
        acqTargetCard: { value: string | null };
        loginGateRelease: { release: (() => Promise<void>) | null };
    },
    object
>({
    testUser: [
        // Empty destructuring `{}` = "this fixture depends on no other
        // fixtures". Playwright requires the (context, use) signature; an
        // empty object is the idiomatic way to say "no dependencies".
        async ({}, use) => {
            const ctx = await setupTestUser();
            await use(ctx);
            await ctx.cleanup();
        },
        { scope: "test" },
    ],
    page: [
        async ({ browser, testUser }, use) => {
            const context = await browser.newContext();
            const page = await context.newPage();
            await page.setViewportSize({ width: 1280, height: 720 });
            try {
                await uiLogin(page, testUser.email, testUser.password);
                await use(page);
            } finally {
                await context.close();
            }
        },
        { scope: "test", timeout: 180_000 },
    ],
    // Training rotation log: each answer appends the front's data-card-id so
    // Then-steps can assert rotation invariants (per-round card sets,
    // direction switch timing) across separate When-steps.
    acqTrainingLog: [
        async ({}, use) => {
            await use([] as string[]);
        },
        { scope: "test" },
    ],
    // Card remembered before a mark-as-known action (replacement asserts).
    acqKnownCard: [
        async ({}, use) => {
            await use({ value: null as string | null });
        },
        { scope: "test" },
    ],
    // Target card of a selective-answers training step (reopen asserts).
    acqTargetCard: [
        async ({}, use) => {
            await use({ value: null as string | null });
        },
        { scope: "test" },
    ],
    // Release callback of the held login request (spinner test): the
    // "held" When-step stores it here, the "released" step calls it —
    // manual gate instead of a fixed sleep, so the in-flight assert
    // cannot flake on timing.
    loginGateRelease: [
        async ({}, use) => {
            await use({ release: null });
        },
        { scope: "test" },
    ],
});

export const { Given, When, Then } = createBdd(test);
