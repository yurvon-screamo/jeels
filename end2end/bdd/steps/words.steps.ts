import { expect } from "@playwright/test";
import { Given, When, Then } from "../fixtures";
import { WordsPage } from "../../pages";

Given('у пользователя есть добавленное слово', async ({ page }) => {
    const wordsPage = new WordsPage(page);
    await wordsPage.goto();
    await wordsPage.expectWordsVisible();
    await wordsPage.openAddModal();
    await wordsPage.enterText("私は本を読みます");
    await wordsPage.analyzeText();
    await wordsPage.selectFirstWord();
    await wordsPage.addSelectedWords();
    await expect(wordsPage.wordsGrid).toBeVisible({ timeout: 10_000 });
});

When('пользователь открывает страницу слов', async ({ page }) => {
    const wordsPage = new WordsPage(page);
    await wordsPage.goto();
    await wordsPage.expectWordsVisible();
});

When('вводит текст {string} для анализа', async ({ page }, text: string) => {
    const wordsPage = new WordsPage(page);
    await wordsPage.openAddModal();
    await wordsPage.enterText(text);
    await wordsPage.analyzeText();
});

When('выбирает первое слово из результатов', async ({ page }) => {
    const wordsPage = new WordsPage(page);
    await wordsPage.selectFirstWord();
});

When('подтверждает добавление выбранных слов', async ({ page }) => {
    const wordsPage = new WordsPage(page);
    await wordsPage.addSelectedWords();
});

When('отменяет добавление', async ({ page }) => {
    const wordsPage = new WordsPage(page);
    await wordsPage.cancelAddModal();
    await expect(wordsPage.drawer).not.toBeVisible();
});

When('пользователь удаляет первое слово', async ({ page }) => {
    const wordsPage = new WordsPage(page);
    await wordsPage.deleteCardByIndex(0);
});

When('пользователь отменяет удаление первого слова', async ({ page }) => {
    const wordsPage = new WordsPage(page);
    const countBefore = await wordsPage.getCardCount();
    await wordsPage.cancelDeleteCardByIndex(0);
    expect(await wordsPage.getCardCount()).toBe(countBefore);
});

Then('слово отображается в сетке слов', async ({ page }) => {
    const wordsPage = new WordsPage(page);
    await expect(wordsPage.wordsGrid).toBeVisible({ timeout: 10_000 });
    await expect(wordsPage.emptyState).not.toBeVisible();
});

Then('отображается пустое состояние', async ({ page }) => {
    const wordsPage = new WordsPage(page);
    await expect(wordsPage.emptyState).toBeVisible();
});

Then('страница слов отображается', async ({ page }) => {
    const wordsPage = new WordsPage(page);
    await wordsPage.expectWordsVisible();
});

When('пользователь ищет слово {string}', async ({ page }, query: string) => {
    const wordsPage = new WordsPage(page);
    await wordsPage.searchInput.fill(query);
    await page.waitForTimeout(500);
});

Then('сетка слов пуста', async ({ page }) => {
    const wordsPage = new WordsPage(page);
    await expect(wordsPage.wordsGrid).not.toBeVisible({ timeout: 10_000 });
});

When('выбирает фильтр слов {string}', async ({ page }, filter: string) => {
    const wordsPage = new WordsPage(page);
    await wordsPage.selectFilter(filter as never);
});

When('пользователь отмечает первое слово как известное', async ({ page }) => {
    const wordsPage = new WordsPage(page);
    await wordsPage.markCardAsKnownByIndex(0);
});

When('пользователь нажимает кнопку избранного первого слова', async ({ page }) => {
    const wordsPage = new WordsPage(page);
    await wordsPage.toggleFavoriteByIndex(0);
});

Then('первое слово отмечено как избранное', async ({ page }) => {
    const wordsPage = new WordsPage(page);
    expect(await wordsPage.isFavorited(0)).toBe(true);
});

When('открывает модальное окно добавления слов', async ({ page }) => {
    const wordsPage = new WordsPage(page);
    await wordsPage.openAddModal();
});

Then('отображается вкладка Anki', async ({ page }) => {
    await expect(page.getByTestId("drawer-tab-anki").or(page.getByText("Anki"))).toBeVisible({ timeout: 10_000 });
});

When('нажимает кнопку перехода к наборам', async ({ page }) => {
    const wordsPage = new WordsPage(page);
    await wordsPage.clickSets();
});

Then('страница наборов отображается', async ({ page }) => {
    await expect(page.getByTestId("sets-page")).toBeVisible({ timeout: 15_000 });
});
