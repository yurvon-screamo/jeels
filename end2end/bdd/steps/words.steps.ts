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
