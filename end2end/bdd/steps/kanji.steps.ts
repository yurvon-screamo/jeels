import { expect } from "@playwright/test";
import { Given, When, Then } from "../fixtures";
import { KanjiPage } from "../../pages";

Given('у пользователя есть добавленное кандзи', async ({ page }) => {
    const kanjiPage = new KanjiPage(page);
    await kanjiPage.goto();
    await expect(kanjiPage.kanjiPage).toBeVisible({ timeout: 15_000 });
    await kanjiPage.addBtn.click();
    await expect(kanjiPage.drawer).toBeVisible({ timeout: 10_000 });
    await kanjiPage.drawerAddBtn.click();
    await expect(kanjiPage.kanjiGrid).toBeVisible({ timeout: 30_000 });
});

When('пользователь открывает страницу кандзи', async ({ page }) => {
    const kanjiPage = new KanjiPage(page);
    await kanjiPage.goto();
    await expect(kanjiPage.kanjiPage).toBeVisible({ timeout: 15_000 });
});

When('открывает добавление кандзи', async ({ page }) => {
    const kanjiPage = new KanjiPage(page);
    await kanjiPage.addBtn.click();
    await expect(kanjiPage.drawer).toBeVisible({ timeout: 10_000 });
});

When('подтверждает добавление кандзи', async ({ page }) => {
    const kanjiPage = new KanjiPage(page);
    await kanjiPage.drawerAddBtn.click();
    await expect(kanjiPage.drawer).not.toBeVisible({ timeout: 30_000 });
});

When('нажимает кнопку выбора всех кандзи', async ({ page }) => {
    const kanjiPage = new KanjiPage(page);
    await kanjiPage.drawerSelectAllBtn.click();
});

Then('кандзи отображается в сетке', async ({ page }) => {
    const kanjiPage = new KanjiPage(page);
    await expect(kanjiPage.kanjiGrid).toBeVisible({ timeout: 30_000 });
    await expect(kanjiPage.emptyState).not.toBeVisible();
});

Then('на странице кандзи отображается пустое состояние', async ({ page }) => {
    const kanjiPage = new KanjiPage(page);
    await expect(kanjiPage.emptyState).toBeVisible();
});

When('пользователь удаляет первое кандзи', async ({ page }) => {
    const kanjiPage = new KanjiPage(page);
    await page.getByTestId("kanji-card-item").first().locator('[data-testid*="delete"]').first().click();
    await expect(kanjiPage.deleteModal).toBeVisible();
    await kanjiPage.deleteConfirmBtn.click();
    await expect(kanjiPage.deleteModal).not.toBeVisible({ timeout: 10_000 });
});

When('пользователь отменяет удаление первого кандзи', async ({ page }) => {
    const kanjiPage = new KanjiPage(page);
    await page.getByTestId("kanji-card-item").first().locator('[data-testid*="delete"]').first().click();
    await expect(kanjiPage.deleteModal).toBeVisible();
    await kanjiPage.deleteCancelBtn.click();
    await expect(kanjiPage.deleteModal).not.toBeVisible();
});

When('нажимает кнопку возврата на кандзи', async ({ page }) => {
    const kanjiPage = new KanjiPage(page);
    await kanjiPage.backBtn.click();
});

When('пользователь отмечает первое кандзи как известное', async ({ page }) => {
    await page.getByTestId("kanji-card-item").first().locator('[data-testid*="mark-as-known"]').click();
});

When('пользователь открывает детали первого кандзи', async ({ page }) => {
    await page.getByTestId("kanji-card-item").first().click();
    await page.waitForURL(/\/kanji\//, { timeout: 10_000 });
});

Then('отображается страница деталей кандзи', async ({ page }) => {
    await expect(page.getByTestId("kanji-detail-page")).toBeVisible({ timeout: 10_000 });
});
