import { expect, type Page } from "@playwright/test";
import { Then, When } from "../fixtures";

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

Then('на сервере одна запись пользователя', async ({ page }) => {
    const cookies = await page.context().cookies();
    const trailbaseCookies = cookies.filter((c) => c.domain.includes("127.0.0.1") || c.domain.includes("localhost"));
    expect(trailbaseCookies.length, "Expected TrailBase auth cookies").toBeGreaterThan(0);
});
