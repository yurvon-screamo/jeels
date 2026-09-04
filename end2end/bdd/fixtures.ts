import { type Page } from "@playwright/test";
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
        cdnRequestLog: string[];
        apiRequestLog: string[];
        secondDevicePage: Page;
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
    // Request URLs observed during a clean-cache app start (the
    // rkyv fast-path scenario): the When-step that drives the fresh
    // browser context appends every network request here, the Then-step
    // asserts the fallback sources were never fetched.
    cdnRequestLog: [
        async ({}, use) => {
            await use([] as string[]);
        },
        { scope: "test" },
    ],
    // URLs of PATCH requests against the user record, observed by the
    // stress scenarios to assert the sync short-circuit (ADR-045): the
    // When-step attaches a request listener AFTER the measurement window
    // is defined, the Then-step counts entries.
    apiRequestLog: [
        async ({}, use) => {
            await use([] as string[]);
        },
        { scope: "test" },
    ],
    // A second, independent browser context logged into the SAME test
    // account — an honest "new device": its IndexedDB partition is empty,
    // so the first login there exercises the remote→local restore path
    // (ADR-045) rather than the logout dance (whose remote-survival is an
    // accident of operation ordering, not a contract). The context stays
    // open for the whole test; later steps navigate its page.
    secondDevicePage: [
        async ({ browser, testUser }, use) => {
            // Own timeout slot (does NOT touch the test deadline): the
            // login on this device covers the multi-minute restore of a
            // large corpus (remote → local, ADR-045) — the project
            // default 60s aborts it mid-restore.
            const context = await browser.newContext({
                baseURL: "http://localhost:1420",
                locale: "ru-RU",
            });
            const page = await context.newPage();
            await page.setViewportSize({ width: 1280, height: 720 });
            await uiLogin(page, testUser.email, testUser.password);
            await use(page);
            await context.close();
        },
        { scope: "test", timeout: 420_000 },
    ],
});

export const { Given, When, Then, After } = createBdd(test);
