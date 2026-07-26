import { expect } from "@playwright/test";
import { When, Then } from "../fixtures";
import { HomePage, GrammarPage } from "../../pages";

When('пользователь открывает главную страницу', async ({ page }) => {
    const homePage = new HomePage(page);
    await homePage.goto();
    await expect(homePage.homePage).toBeVisible({ timeout: 15_000 });
});

Then('главная страница отображается', async ({ page }) => {
    const homePage = new HomePage(page);
    await expect(homePage.homePage).toBeVisible();
});

When('нажимает навигацию к словам', async ({ page }) => {
    const homePage = new HomePage(page);
    await homePage.sidebarWords.click();
});

When('нажимает навигацию к грамматике', async ({ page }) => {
    const homePage = new HomePage(page);
    await homePage.sidebarGrammar.click();
});

Then('страница грамматики отображается', async ({ page }) => {
    const grammarPage = new GrammarPage(page);
    await expect(grammarPage.grammarPage).toBeVisible({ timeout: 15_000 });
});

Then('отображается кнопка начала урока', async ({ page }) => {
    const homePage = new HomePage(page);
    await expect(homePage.lessonButton).toBeVisible();
});

Then('отображается карточка прогресса JLPT', async ({ page }) => {
    const homePage = new HomePage(page);
    await expect(homePage.jlptProgress).toBeVisible();
});

Then('отображается обзор активности', async ({ page }) => {
    const homePage = new HomePage(page);
    await expect(homePage.todayOverview).toBeVisible();
});

Then('отображается недавняя активность', async ({ page }) => {
    await expect(page.getByTestId("home-recent-study")).toBeVisible({ timeout: 10_000 });
});

Then('отображается приветственная карточка', async ({ page }) => {
    const homePage = new HomePage(page);
    await expect(homePage.welcomeCard).toBeVisible();
});

When('раскрывает детали прогресса JLPT', async ({ page }) => {
    const homePage = new HomePage(page);
    await homePage.jlptProgress.click();
});

Then('отображаются категории прогресса', async ({ page }) => {
    await expect(page.getByTestId(/jlpt.*categor/)).toBeVisible({ timeout: 5_000 });
});
