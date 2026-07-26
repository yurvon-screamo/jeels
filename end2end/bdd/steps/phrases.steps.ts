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

Then('карточки фраз имеют непустой текст', async ({ page }) => {
    const phrasesPage = new PhrasesPage(page);
    await expect(phrasesPage.phrasesGrid).toBeVisible({ timeout: 30_000 });
    const firstCard = phrasesPage.cardItem.first();
    await expect(firstCard).toBeVisible();
    const text = await firstCard.textContent();
    expect(text?.trim().length ?? 0).toBeGreaterThan(0);
});

When('ищет фразы {string}', async ({ page }, query: string) => {
    const phrasesPage = new PhrasesPage(page);
    await phrasesPage.searchPhrases(query);
    await page.waitForTimeout(500);
});

Then('на странице фраз нет карточек', async ({ page }) => {
    const phrasesPage = new PhrasesPage(page);
    await expect(phrasesPage.phrasesGrid).not.toBeVisible({ timeout: 10_000 });
});

When('удаляет первую фразу', async ({ page }) => {
    await page.getByTestId("phrase-card-item").first().locator('[data-testid*="delete"]').first().click();
});

Then('отображается сообщение о подтверждении удаления', async ({ page }) => {
    const phrasesPage = new PhrasesPage(page);
    await expect(phrasesPage.deleteModal).toBeVisible();
});
