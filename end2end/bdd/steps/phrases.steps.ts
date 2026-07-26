import { expect } from "@playwright/test";
import { When, Then } from "../fixtures";
import { PhrasesPage } from "../../pages";

When('пользователь открывает страницу фраз', async ({ page }) => {
    const phrasesPage = new PhrasesPage(page);
    await phrasesPage.goto();
    await phrasesPage.expectPhrasesVisible();
});

Then('на странице фраз отображается пустое состояние', async ({ page }) => {
    const phrasesPage = new PhrasesPage(page);
    await expect(phrasesPage.emptyState).toBeVisible();
});

Then('отображается поле поиска фраз', async ({ page }) => {
    const phrasesPage = new PhrasesPage(page);
    await expect(phrasesPage.searchInput).toBeVisible();
});

Then('отображается вкладка фраз в навигации', async ({ page }) => {
    await expect(page.getByTestId("nav-phrases")).toBeVisible({ timeout: 15_000 });
});

When('нажимает кнопку возврата с фраз', async ({ page }) => {
    const phrasesPage = new PhrasesPage(page);
    await phrasesPage.backButton.click();
});
