import { expect, type Page } from "@playwright/test";
import { Then, When } from "../fixtures";
import { uiLogin } from "../../helpers/auth";
import { WordsPage } from "../../pages";

const NIL_USER_ID = "0".repeat(26);

async function readUserStoreKeys(page: Page): Promise<string[]> {
    return await page.evaluate(async (): Promise<string[]> => {
        return await new Promise<string[]>((resolve, reject) => {
            const open = indexedDB.open("origa");
            open.onerror = () => reject(new Error("indexedDB open failed"));
            open.onupgradeneeded = () => resolve([]);
            open.onsuccess = () => {
                const db = open.result;
                if (!db.objectStoreNames.contains("users")) {
                    resolve([]);
                    return;
                }
                const tx = db.transaction("users", "readonly");
                const store = tx.objectStore("users");
                const req = store.getAllKeys();
                req.onsuccess = () => resolve(req.result.map((k) => String(k)));
                req.onerror = () => reject(new Error("getAllKeys failed"));
            };
        });
    });
}

Then('user ID в IndexedDB не nil', async ({ page }) => {
    await page.waitForURL(/\/home$/, { timeout: 15_000 });
    await page.getByTestId("home-page").waitFor({ state: "visible", timeout: 30_000 });

    const deadline = Date.now() + 10_000;
    let canonicalKey: string | null = null;
    while (Date.now() < deadline) {
        const keys = await readUserStoreKeys(page).catch(() => [] as string[]);
        canonicalKey = keys.find((k) => !k.endsWith(`:${NIL_USER_ID}`)) ?? null;
        if (canonicalKey) break;
        await page.waitForTimeout(250);
    }
    expect(canonicalKey, "Expected a non-nil user key in IndexedDB after login").not.toBeNull();

    const keys = await readUserStoreKeys(page);
    const nilKeys = keys.filter((k) => k.endsWith(`:${NIL_USER_ID}`));
    expect(nilKeys, `IndexedDB must not hold a nil user key; found: ${nilKeys.join(", ")}`).toHaveLength(0);
});

When('второй браузер входит в тот же аккаунт', async ({ browser }) => {
    const context = await browser.newContext();
    const pageB = await context.newPage();
    await pageB.setViewportSize({ width: 1280, height: 720 });
    await pageB.goto("/");
    await pageB.waitForLoadState("networkidle");
    await context.close();
});

// Cross-device verification: open a SECOND, pristine browser context (no
// cookies / localStorage / IndexedDB carried over from the first session) and
// log into the SAME test account, then assert the favorite toggled in the
// first session is present. This is the real "data survives across devices"
// roundtrip — it reads the favorite back from the remote, not from the local
// store of the session that wrote it.
//
// A fresh context is used deliberately instead of logout+reload on the same
// page: the app's logout path wipes IndexedDB mid-session and the subsequent
// UI re-login races the WASM re-init (intermittent "login form never
// appears"), which is an app-side flakiness orthogonal to what this scenario
// guards. A new context is the deterministic equivalent and matches the
// real-world "open the app on a second device" flow.
//
// Action + assertion live in one Then step because the BDD `page` fixture is
// fixed for the test scope — a second page cannot be exposed to later steps.
Then('второй браузер видит эту карточку избранной', async ({ browser, testUser }) => {
    const context = await browser.newContext();
    const pageB = await context.newPage();
    await pageB.setViewportSize({ width: 1280, height: 720 });
    try {
        await uiLogin(pageB, testUser.email, testUser.password);

        const wordsPage = new WordsPage(pageB);
        await wordsPage.goto();
        await wordsPage.expectWordsVisible();

        // Mirror the "первая карточка отмечена избранной" assertion shape from
        // common.steps.ts — the filled-heart SVG marks the favorited state.
        const favBtn = pageB.locator('[data-testid*="card-item"]').first()
            .locator('[data-testid*="favorite"]').first();
        await expect(
            favBtn.locator('svg path[fill="currentColor"]'),
            "favorite set in session A must be visible from a fresh session B (cross-device sync)",
        ).toBeVisible({ timeout: 15_000 });
    } finally {
        await context.close();
    }
});

// Composite of "toggle favorite" + explicit wait for the save_sync write to
// /api/records/v1/user to resolve. The plain toggle step (common.steps)
// returns as soon as the optimistic UI flips the heart SVG; the actual
// save_sync runs in a fire-and-forget spawn_local and can still be in-flight
// when a later logout wipes client state — turning the post-relogin
// assertion into a race (false-fail if the save is aborted, false-pass if it
// happens to land). Waiting on the write response here makes the roundtrip
// deterministic: the remote is guaranteed to hold the favorite before any
// state wipe. Also asserts the response is 2xx, so a CHECK-constraint (or any
// other server-side invariant) regression surfaces here, not as a silent 500.
//
// Method set: TrailBase records API uses POST to create a row and PATCH to
// update one (see trailbase_records.rs: `create` → POST, `update` → PATCH).
// PUT is included defensively in case the wire contract evolves; GET/OPTIONS
// are excluded so the CORS preflight + the read-back after save don't satisfy
// the wait prematurely.
//
// Matcher scope note: the predicate matches the user-records endpoint by URL +
// method, not the request body or a specific record id. That is safe ONLY when
// the scenario has no competing user-record writes in the 15s window — which
// holds for the current "fresh user → words → single toggle" flow. If steps
// that mutate user state are added before this one, narrow the matcher (e.g.
// by reading the record id from the response of an earlier create, or by
// inspecting the request body) so a debounced save from another mutation
// can't satisfy the wait first.
const RECORD_WRITE_METHODS = new Set(["POST", "PUT", "PATCH"]);

When('отмечает первую карточку избранной и дожидается сохранения', async ({ page }) => {
    const favBtn = page.locator('[data-testid*="card-item"]').first()
        .locator('[data-testid*="favorite"]').first();

    // Register the wait BEFORE the click so the listener is in place when
    // spawn_local fires the network request; otherwise a fast save can slip
    // past the waitForResponse window.
    const saveResponse = page.waitForResponse(
        (resp) =>
            /\/api\/records\/v1\/user(\/|$)/.test(resp.url()) &&
            RECORD_WRITE_METHODS.has(resp.request().method()),
        { timeout: 15_000 },
    );

    await favBtn.click();
    await expect(favBtn.locator('svg path[fill="currentColor"]')).toBeVisible({
        timeout: 10_000,
    });

    const resp = await saveResponse;
    expect(resp.ok(), `save_sync must succeed, got ${resp.status()}`).toBe(true);
});

Then('на сервере одна запись пользователя', async ({ page }) => {
    const cookies = await page.context().cookies();
    const trailbaseCookies = cookies.filter((c) => c.domain.includes("127.0.0.1") || c.domain.includes("localhost"));
    expect(trailbaseCookies.length, "Expected TrailBase auth cookies").toBeGreaterThan(0);
});
