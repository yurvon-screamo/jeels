import { test as base } from '@playwright/test';
import { setupTestUser, uiLogin } from '../helpers/auth';
const test = base.extend({});
test.setTimeout(300000);
test('jlpt n1 drawer open/close timing', async ({ browser }) => {
    const ctx = await setupTestUser();
    const context = await browser.newContext();
    const page = await context.newPage();
    try {
        const t0 = Date.now();
        await uiLogin(page, ctx.email, ctx.password);
        console.log('login ms:', Date.now() - t0);
        await page.goto('/sets');
        await page.getByTestId('sets-page').waitFor({ timeout: 90000 });
        console.log('sets ms:', Date.now() - t0);
        await page.getByRole('button', { name: 'N1', exact: true }).first().click();
        await page.waitForTimeout(2000);
        console.log('n1 filter ms:', Date.now() - t0);
        const card = page.getByTestId('sets-card-item').filter({ hasText: 'JLPT' }).first();
        await card.getByTestId('sets-card-import-btn').first().click();
        const drawer = page.getByTestId('sets-import-drawer');
        await drawer.waitFor({ state: 'visible', timeout: 60000 });
        console.log('drawer open ms:', Date.now() - t0);
        await page.getByTestId('sets-drawer-item').first().waitFor({ state: 'visible', timeout: 60000 });
        console.log('first item ms:', Date.now() - t0);
        const cnt = await page.getByTestId('sets-drawer-item').count();
        console.log('items:', cnt);
        const tc = Date.now();
        await page.getByTestId('sets-drawer-cancel-btn').click();
        await drawer.waitFor({ state: 'hidden', timeout: 120000 });
        console.log('drawer close ms:', Date.now() - tc);
        console.log('ALIVE:', await page.getByTestId('sets-page').count());
    } finally {
        await context.close();
        await ctx.cleanup();
    }
});
