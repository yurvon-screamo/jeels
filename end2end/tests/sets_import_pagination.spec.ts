import { expect, test as base } from '@playwright/test';
import { setupTestUser, uiLogin } from '../helpers/auth';

const test = base.extend({});

test.setTimeout(240000);

async function openSets(page: import('@playwright/test').Page) {
    await page.goto('/sets');
    await page.getByTestId('sets-page').waitFor({ timeout: 90_000 });
}

async function openDrawerAndAssertPaginated(page: import('@playwright/test').Page) {
    const drawer = page.getByTestId('sets-import-drawer');
    await drawer.waitFor({ state: 'visible', timeout: 30_000 });
    await expect(page.getByTestId('sets-drawer-item').first()).toBeVisible({
        timeout: 60_000,
    });
    const initial = await page.getByTestId('sets-drawer-item').count();
    expect(initial).toBeLessThanOrEqual(150);
    await expect(page.getByTestId('sets-drawer-load-more-btn')).toBeVisible({
        timeout: 30_000,
    });
    return { drawer, initial };
}

/**
 * Regression (iOS black-screen OOM): selecting a whole level (N3 → 236 sets,
 * ~8.5k words) used to render every preview word at once in the import
 * drawer. On iOS WKWebView (~1.5 GB jetsam limit) that killed the page to a
 * black screen before the word list appeared. The drawer now paginates the
 * preview (first 100 words + a load-more button); the import itself still
 * covers all selected words.
 */
test('N3 level multi-select preview stays paginated and the page survives', async ({ browser }) => {
    const ctx = await setupTestUser();
    const context = await browser.newContext();
    const page = await context.newPage();
    try {
        await uiLogin(page, ctx.email, ctx.password);
        await openSets(page);
        await page.getByRole('button', { name: 'N3', exact: true }).first().click();
        await page.waitForTimeout(2_000);

        const boxes = page.locator('[data-testid="sets-card-item"] .checkbox-container');
        const boxCount = await boxes.count();
        expect(boxCount).toBeGreaterThan(0);
        for (let i = 0; i < boxCount; i++) {
            await boxes.nth(i).click({ timeout: 10_000 });
        }
        await page.getByTestId('sets-import-selected-btn').click();

        const { drawer, initial } = await openDrawerAndAssertPaginated(page);
        const loadMore = page.getByTestId('sets-drawer-load-more-btn');
        await loadMore.click();
        await page.waitForTimeout(1_000);
        const grown = await page.getByTestId('sets-drawer-item').count();
        expect(grown).toBeGreaterThan(initial);

        await page.getByTestId('sets-drawer-cancel-btn').click();
        await expect(drawer).not.toBeVisible({ timeout: 30_000 });
        await expect(page.getByTestId('sets-card-item').first()).toBeVisible({
            timeout: 30_000,
        });
    } finally {
        await context.close();
        await ctx.cleanup();
    }
});

test('JLPT N1 single set (3331 words) preview stays paginated', async ({ browser }) => {
    const ctx = await setupTestUser();
    const context = await browser.newContext();
    const page = await context.newPage();
    try {
        await uiLogin(page, ctx.email, ctx.password);
        await openSets(page);
        await page.getByRole('button', { name: 'N1', exact: true }).first().click();
        await page.waitForTimeout(2_000);
        const jlptCard = page
            .getByTestId('sets-card-item')
            .filter({ hasText: 'JLPT' })
            .first();
        await jlptCard.getByTestId('sets-card-import-btn').first().click();
        const { drawer } = await openDrawerAndAssertPaginated(page);
        await page.getByTestId('sets-drawer-cancel-btn').click();
        await expect(drawer).not.toBeVisible({ timeout: 30_000 });
    } finally {
        await context.close();
        await ctx.cleanup();
    }
});

test('Minna N5 multi-select (25 sets) preview stays paginated', async ({ browser }) => {
    const ctx = await setupTestUser();
    const context = await browser.newContext();
    const page = await context.newPage();
    try {
        await uiLogin(page, ctx.email, ctx.password);
        await openSets(page);
        await page.getByRole('button', { name: 'N5', exact: true }).first().click();
        await page.waitForTimeout(1_500);
        await page
            .getByRole('button', { name: 'Minna no Nihongo', exact: true })
            .first()
            .click();
        await page.waitForTimeout(1_500);
        const boxes = page.locator('[data-testid="sets-card-item"] .checkbox-container');
        const boxCount = await boxes.count();
        expect(boxCount).toBeGreaterThan(0);
        for (let i = 0; i < boxCount; i++) {
            await boxes.nth(i).click({ timeout: 10_000 });
        }
        await page.getByTestId('sets-import-selected-btn').click();
        const { drawer } = await openDrawerAndAssertPaginated(page);
        await page.getByTestId('sets-drawer-cancel-btn').click();
        await expect(drawer).not.toBeVisible({ timeout: 30_000 });
    } finally {
        await context.close();
        await ctx.cleanup();
    }
});
