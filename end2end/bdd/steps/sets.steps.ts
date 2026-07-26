import { expect } from "@playwright/test";
import { When, Then } from "../fixtures";
import { SetsPage } from "../../pages";

When('пользователь открывает страницу наборов', async ({ page }) => {
    const setsPage = new SetsPage(page);
    await setsPage.goto();
    await setsPage.expectSetsVisible();
    await setsPage.waitForLoad();
});

Then('отображается список доступных наборов', async ({ page }) => {
    const setsPage = new SetsPage(page);
    expect(await setsPage.getSetCardCount()).toBeGreaterThan(0);
    expect(await setsPage.getImportedCardCount()).toBe(0);
});

When('импортирует первый набор', async ({ page }) => {
    const setsPage = new SetsPage(page);
    const countBefore = await setsPage.getImportedCardCount();
    await setsPage.clickImportOnCard(0);
    await setsPage.waitForDrawerWords();
    await setsPage.importFromDrawer();
    await setsPage.waitForLoad();
    const countAfter = await setsPage.getImportedCardCount();
    expect(countAfter).toBeGreaterThanOrEqual(countBefore);
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
