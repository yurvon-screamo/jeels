import { expect } from "@playwright/test";
import { When, Then } from "../fixtures";
import { HomePage, LoginPage } from "../../pages";

// EN-копия для точечных проверок локали (locales/en.json: login.subtitle,
// home.words/home.phrases). Текст здесь — сам объект проверки, поэтому
// допустим текстовый ассерт вместо data-testid.
const EN_LOGIN_SUBTITLE = /study japanese/i;

When('переключает язык интерфейса на английский', async ({ page }) => {
    const loginPage = new LoginPage(page);
    await loginPage.englishToggle.waitFor({ state: "visible", timeout: 10_000 });
    await loginPage.englishToggle.click();
});

Then('подзаголовок страницы входа на английском', async ({ page }) => {
    const loginPage = new LoginPage(page);
    await expect(loginPage.subtitle).toHaveText(EN_LOGIN_SUBTITLE, { timeout: 10_000 });
});

Then('боковая навигация на английском', async ({ page }) => {
    const homePage = new HomePage(page);
    await expect(homePage.sidebar).toBeVisible({ timeout: 30_000 });
    await expect(homePage.sidebar).toContainText("Words");
    await expect(homePage.sidebar).toContainText("Phrases");
});
