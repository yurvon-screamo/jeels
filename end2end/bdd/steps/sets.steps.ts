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
    await setsPage.expectImportedBadgeWithoutReload();
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
    await setsPage.clickImportOnCard(0);
    await setsPage.waitForDrawerWords();
    await setsPage.importFromDrawer();
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

Then('набор отображается как импортированный без перезагрузки страницы', async ({ page }) => {
    const setsPage = new SetsPage(page);
    await setsPage.expectImportedBadgeWithoutReload();
});

Then('отображается уведомление об успешном импорте', async ({ page }) => {
    const setsPage = new SetsPage(page);
    await setsPage.expectImportToastVisible();
});

When('открывает окно предпросмотра первого набора', async ({ page }) => {
    const setsPage = new SetsPage(page);
    await setsPage.openFirstSetPreview();
});

Then('кнопки импорта и отмены полностью видны на экране', async ({ page }) => {
    const setsPage = new SetsPage(page);
    await setsPage.waitForDrawerWords();
    await setsPage.expectDrawerActionsInViewport();
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


// --- Пагинация предпросмотра импорта (iOS OOM regression) ---
// Rendering every preview word at once (level multi-select loads 8000+
// items) blew the WKWebView jetsam limit and killed the page to a black
// screen; the drawer paginates the preview instead.

When('выбирает все наборы уровня', async ({ page }) => {
    const setsPage = new SetsPage(page);
    await setsPage.selectAllSets();
});

When('нажимает кнопку импорта выбранных', async ({ page }) => {
    const setsPage = new SetsPage(page);
    await setsPage.importSelectedBtn.click();
});

Then('открывается окно предпросмотра со словами', async ({ page }) => {
    const setsPage = new SetsPage(page);
    await expect(setsPage.drawer).toBeVisible({ timeout: 30_000 });
    await expect(setsPage.drawerWordItems.first()).toBeVisible({ timeout: 60_000 });
});

Then('количество отображённых слов ограничено пагинацией', async ({ page }) => {
    const setsPage = new SetsPage(page);
    const count = await setsPage.drawerWordItems.count();
    // PREVIEW_PAGE_SIZE (100) plus a slack for the load-more click below
    expect(count).toBeLessThanOrEqual(150);
});

Then('кнопка загрузки ещё слов предпросмотра отображается', async ({ page }) => {
    const setsPage = new SetsPage(page);
    await expect(setsPage.drawerLoadMoreBtn).toBeVisible({ timeout: 30_000 });
});

When('нажимает кнопку загрузки ещё слов предпросмотра', async ({ page }) => {
    const setsPage = new SetsPage(page);
    await setsPage.drawerLoadMoreBtn.click();
});

Then('отображается больше слов предпросмотра', async ({ page }) => {
    const setsPage = new SetsPage(page);
    const count = await setsPage.drawerWordItems.count();
    expect(count).toBeGreaterThan(100);
});

When('отменяет предпросмотр', async ({ page }) => {
    const setsPage = new SetsPage(page);
    await setsPage.drawerCancelBtn.click();
    await expect(setsPage.drawer).not.toBeVisible({ timeout: 30_000 });
});
