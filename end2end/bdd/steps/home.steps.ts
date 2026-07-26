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

When('нажимает кнопку начала урока', async ({ page }) => {
    const homePage = new HomePage(page);
    await homePage.startLesson();
});

When('нажимает нижнюю вкладку слов', async ({ page }) => {
    await page.getByTestId("bottom-tab-tab-words").click();
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

Then('приветственная карточка содержит имя пользователя', async ({ page }) => {
    const homePage = new HomePage(page);
    await expect(homePage.welcomeCard).toBeVisible();
    const text = await homePage.welcomeCard.textContent();
    expect(text?.trim().length ?? 0).toBeGreaterThan(0);
});

Then('отображается график активности', async ({ page }) => {
    await expect(page.getByTestId(/activity.*chart|chart.*activity/)).toBeVisible({ timeout: 10_000 });
});

Then('отображается нижняя панель навигации', async ({ page }) => {
    await expect(page.locator(".bottom-tab-bar")).toBeVisible({ timeout: 10_000 });
});
