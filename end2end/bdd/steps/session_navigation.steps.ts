import { expect } from "@playwright/test";
import { When, Then } from "../fixtures";
import { HomePage, KanjiPage } from "../../pages";

When('пользователь переходит на страницу слов', async ({ page }) => {
    await page.goto("/words");
    await page.waitForLoadState("networkidle");
});

When('пользователь перезагружает страницу', async ({ page }) => {
    await page.reload();
    await page.waitForLoadState("networkidle");
});

When('возвращается на главную страницу', async ({ page }) => {
    const homePage = new HomePage(page);
    await homePage.goto();
});

When('нажимает навигацию к кандзи', async ({ page }) => {
    const homePage = new HomePage(page);
    await homePage.sidebarKanji.click();
});

Then('страница кандзи отображается', async ({ page }) => {
    const kanjiPage = new KanjiPage(page);
    await expect(kanjiPage.kanjiPage).toBeVisible({ timeout: 15_000 });
});

Then('отображается боковая панель навигации', async ({ page }) => {
    const homePage = new HomePage(page);
    await expect(homePage.sidebar).toBeVisible();
});
