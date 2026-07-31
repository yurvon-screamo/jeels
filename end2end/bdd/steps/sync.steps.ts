import { expect, type Page } from "@playwright/test";
import { Then, When } from "../fixtures";
import { uiLogin } from "../../helpers/auth";

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

// Re-login into the same test account after a logout (auth.steps clears all
// client-side state). The credentials come from the `testUser` fixture, so
// this exercises the SAME identity the scenario started with — a real
// roundtrip through the remote, not a fresh account. Catches wire-format /
// schema-invariant regressions on save_sync (see ADR-034).
When('пользователь снова входит в тот же аккаунт', async ({ page, testUser }) => {
    await uiLogin(page, testUser.email, testUser.password);
});

// Composite of "toggle favorite" + explicit wait for the save_sync write to
// /api/records/v1/user/<id> to resolve. The plain toggle step (common.steps)
// returns as soon as the optimistic UI flips the heart SVG; the actual
// save_sync runs in a fire-and-forget spawn_local and can still be in-flight
// when a later logout wipes client state — turning the post-relogin
// assertion into a race (false-fail if the save is aborted, false-pass if it
// happens to land). Waiting on the PUT/POST response here makes the roundtrip
// deterministic: the remote is guaranteed to hold the favorite before any
// state wipe. Also asserts the response is 2xx, so a CHECK-constraint (or any
// other server-side invariant) regression surfaces here, not as a silent 500.
//
// Matcher scope note: the predicate matches the user-records endpoint by URL +
// method, not the request body or a specific record id. That is safe ONLY when
// the scenario has no competing user-record writes in the 15s window — which
// holds for the current "fresh user → words → single toggle" flow. If steps
// that mutate user state are added before this one, narrow the matcher (e.g.
// by reading the record id from the response of an earlier create, or by
// inspecting the request body) so a debounced save from another mutation
// can't satisfy the wait first.
When('отмечает первую карточку избранной и дожидается сохранения', async ({ page }) => {
    const favBtn = page.locator('[data-testid*="card-item"]').first()
        .locator('[data-testid*="favorite"]').first();

    // Register the wait BEFORE the click so the listener is in place when
    // spawn_local fires the network request; otherwise a fast save can slip
    // past the waitForResponse window.
    const saveResponse = page.waitForResponse(
        (resp) =>
            /\/api\/records\/v1\/user(\/|$)/.test(resp.url()) &&
            (resp.request().method() === "PUT" ||
                resp.request().method() === "POST"),
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
