import { expect } from "@playwright/test";
import { Given, When, Then } from "../fixtures";
import { SetsPage } from "../../pages";

When('пользователь открывает страницу наборов', async ({ page }) => {
    const setsPage = new SetsPage(page);
    await setsPage.goto();
    await setsPage.expectSetsVisible();
    await setsPage.waitForLoad();
});

Given('у пользователя есть импортированный набор', async ({ page }) => {
    const setsPage = new SetsPage(page);
    await setsPage.goto();
    await setsPage.expectSetsVisible();
    await setsPage.waitForLoad();
    await setsPage.clickImportOnCard(0);
    await setsPage.waitForDrawerWords();
    await setsPage.importFromDrawer();
    // The sets list signal doesn't re-fetch automatically after the drawer
    // closes; a reload surfaces the reimport-button state.
    await page.reload();
    await setsPage.waitForLoad();
    const imported = await setsPage.getImportedCardCount();
    expect(imported).toBeGreaterThan(0);
});

Then('отображается список доступных наборов', async ({ page }) => {
    const setsPage = new SetsPage(page);
    expect(await setsPage.getSetCardCount()).toBeGreaterThan(0);
    expect(await setsPage.getImportedCardCount()).toBe(0);
});

Then('отображается импортированный набор', async ({ page }) => {
    const setsPage = new SetsPage(page);
    expect(await setsPage.getImportedCardCount()).toBeGreaterThan(0);
});

When('импортирует первый набор', async ({ page }) => {
    const setsPage = new SetsPage(page);
    const countBefore = await setsPage.getImportedCardCount();
    await setsPage.clickImportOnCard(0);
    await setsPage.waitForDrawerWords();
    await setsPage.importFromDrawer();
    // The import use-case persists the imported flag on the server, but the
    // sets list signal in the UI doesn't re-fetch automatically after the
    // drawer closes — a reload forces the re-fetch and surfaces the
    // reimport-button state.
    await page.reload();
    await setsPage.waitForLoad();
    const countAfter = await setsPage.getImportedCardCount();
    expect(countAfter).toBeGreaterThan(countBefore);
});

When('отменяет импорт первого набора', async ({ page }) => {
    const setsPage = new SetsPage(page);
    const countBefore = await setsPage.getImportedCardCount();
    await setsPage.clickImportOnCard(0);
    await setsPage.waitForDrawerWords();
    await setsPage.cancelImportFromDrawer();
    await expect(setsPage.drawer).not.toBeVisible();
    expect(await setsPage.getImportedCardCount()).toBe(countBefore);
});

Then('набор отображается как импортированный', async ({ page }) => {
    const setsPage = new SetsPage(page);
    expect(await setsPage.getImportedCardCount()).toBeGreaterThan(0);
});

Then('ни один набор не импортирован', async ({ page }) => {
    const setsPage = new SetsPage(page);
    expect(await setsPage.getImportedCardCount()).toBe(0);
});

When('ищет наборы {string}', async ({ page }, query: string) => {
    const setsPage = new SetsPage(page);
    await setsPage.searchSets(query);
    await page.waitForTimeout(500);
});

Then('список наборов пуст', async ({ page }) => {
    const setsPage = new SetsPage(page);
    expect(await setsPage.getSetCardCount()).toBe(0);
});

When('выбирает первый набор', async ({ page }) => {
    const setsPage = new SetsPage(page);
    await setsPage.selectSetCheckbox(0);
});

When('выбирает второй набор', async ({ page }) => {
    const setsPage = new SetsPage(page);
    await setsPage.selectSetCheckbox(1);
});

Then('отображается кнопка импорта выбранных', async ({ page }) => {
    const setsPage = new SetsPage(page);
    await expect(setsPage.importSelectedBtn).toBeVisible();
});

When('отменяет выбор наборов', async ({ page }) => {
    const setsPage = new SetsPage(page);
    await setsPage.cancelSelection();
});

Then('кнопка импорта выбранных скрыта', async ({ page }) => {
    const setsPage = new SetsPage(page);
    await expect(setsPage.importSelectedBtn).not.toBeVisible();
});

When('фильтрует наборы по уровню {string}', async ({ page }, level: string) => {
    // Level filter tags use the testid pattern sets-level-<level-lowercase>
    // (e.g. sets-level-n5, sets-level-n4, ...).
    const filterBtn = page.getByTestId(`sets-level-${level.toLowerCase()}`);
    await filterBtn.click();
});

When('фильтрует наборы по типу {string}', async ({ page }, typeId: string) => {
    // SetType IDs come from the CDN manifest (e.g. "Jlpt", "DuolingoEn").
    // Filter buttons render as data-testid="sets-type-<TypeId>".
    const filterBtn = page.getByTestId(`sets-type-${typeId}`);
    await filterBtn.click();
});

Then('отображаются наборы уровня {string}', async ({ page }, level: string) => {
    const setsPage = new SetsPage(page);
    const count = await setsPage.getSetCardCount();
    expect(count, `expected sets for level ${level}`).toBeGreaterThan(0);
});

When('фильтрует наборы по статусу {string}', async ({ page }, status: string) => {
    // Status filter tags use the testid pattern sets-import-<status>
    // (e.g. sets-import-imported, sets-import-new, sets-import-all).
    const filterBtn = page.getByTestId(`sets-import-${status}`);
    await filterBtn.click();
});

Then('отображается больше наборов', async ({ page }) => {
    const setsPage = new SetsPage(page);
    expect(await setsPage.getSetCardCount()).toBeGreaterThan(0);
});

When('пользователь эмулирует отсутствие сети', async ({ page }) => {
    // Clear Cache API so CDN requests are not served from cache by
    // CacheFirstCdnProvider — previous scenarios populate it.
    await page.evaluate(() => {
        return caches.keys().then((keys) =>
            Promise.all(keys.map((k) => caches.delete(k)))
        );
    });

    // Block CDN requests to simulate network failure for sets data.
    await page.route('**/well_known_set/**', (route) =>
        route.abort('failed')
    );
    await page.route('**/well_known_types_meta.json', (route) =>
        route.abort('failed')
    );
});

Then('отображается сообщение об отсутствии соединения', async ({ page }) => {
    const setsPage = new SetsPage(page);
    await expect(setsPage.offlineError).toBeVisible({ timeout: 15_000 });
});
