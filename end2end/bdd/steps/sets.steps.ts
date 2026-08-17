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
