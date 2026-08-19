import { expect, test as base } from '@playwright/test';
import { setupTestUser, uiLogin } from '../helpers/auth';

/**
 * Regression (iOS black-screen OOM): selecting a whole level (N3 → 236 sets,
 * ~8.5k words) used to render every preview word at once in the import
 * drawer. On iOS WKWebView (~1.5 GB jetsam limit) that killed the page to a
 * black screen before the word list appeared. The drawer now paginates the
 * preview (first PREVIEW_PAGE_SIZE words + a load-more button); the import
 * itself still covers all selected words.
 */
const test = base.extend({});

test.setTimeout(300000);
test('large level import preview stays paginated and the page survives', async ({ browser }) => {
    const ctx = await setupTestUser();
    const context = await browser.newContext();
    const page = await context.newPage();
    try {
    await uiLogin(page, ctx.email, ctx.password);
    await page.goto('/sets');
    await page.getByTestId('sets-page').waitFor({ timeout: 90_000 });

    await page.getByRole('button', { name: 'N3', exact: true }).first().click();
    await page.waitForTimeout(2_000);

    // select every set of the level, then "import selected"
    const boxes = page.locator('[data-testid="sets-card-item"] .checkbox-container');
    const boxCount = await boxes.count();
    expect(boxCount).toBeGreaterThan(0);
    for (let i = 0; i < boxCount; i++) {
        await boxes.nth(i).click({ timeout: 10_000 });
    }
    await page.getByTestId('sets-import-selected-btn').click();

    const drawer = page.getByTestId('sets-import-drawer');
    await drawer.waitFor({ state: 'visible', timeout: 30_000 });

    // the preview list must appear (not a dead/black page) …
    const firstItem = page.getByTestId('sets-drawer-item').first();
    await expect(firstItem).toBeVisible({ timeout: 60_000 });

    // … and with thousands of words the load-more control must be there,
    // capping the initially rendered item count.
    const loadMore = page.getByTestId('sets-drawer-load-more-btn');
    await expect(loadMore).toBeVisible({ timeout: 30_000 });
    const initialItems = await page.getByTestId('sets-drawer-item').count();
    expect(initialItems).toBeLessThanOrEqual(150);

    // load-more grows the visible list
    await loadMore.click();
    await page.waitForTimeout(1_000);
    const grownItems = await page.getByTestId('sets-drawer-item').count();
    expect(grownItems).toBeGreaterThan(initialItems);

    // and the whole flow must leave the page alive
    await page.getByTestId('sets-drawer-cancel-btn').click();
    await expect(drawer).not.toBeVisible({ timeout: 30_000 });
    await expect(page.getByTestId('sets-card-item').first()).toBeVisible({ timeout: 30_000 });
    } finally {
        await context.close();
        await ctx.cleanup();
    }
});
