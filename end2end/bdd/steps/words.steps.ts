import path from "path";
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
    const countBefore = await wordsPage.getCardCount();
    await wordsPage.deleteCardByIndex(0);
    await expect.poll(() => wordsPage.getCardCount()).toBe(countBefore - 1);
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
    await expect(wordsPage.emptyState).toBeVisible({ timeout: 10_000 });
});

When('выбирает фильтр слов {string}', async ({ page }, filter: string) => {
    const filterMap: Record<string, string> = {
        "all": "Все", "new": "Новые", "hard": "Сложные",
        "learning": "В процессе", "learned": "Изученные",
    };
    const wordsPage = new WordsPage(page);
    await wordsPage.selectFilter(filterMap[filter] as never);
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

When('нажимает кнопку возврата на главную', async ({ page }) => {
    await page.goto("/home");
    await page.waitForURL(/\/home$/, { timeout: 10_000 });
});

When('переключает на вкладку Anki', async ({ page }) => {
    const wordsPage = new WordsPage(page);
    await wordsPage.switchToAnkiTab();
});

Then('отображается зона перетаскивания файлов Anki', async ({ page }) => {
    const wordsPage = new WordsPage(page);
    await expect(wordsPage.ankiDropZone).toBeVisible({ timeout: 10_000 });
});

When('загружает неверный файл Anki', async ({ page }) => {
    const wordsPage = new WordsPage(page);
    await wordsPage.ankiFileInput.setInputFiles({
        name: "invalid.txt",
        mimeType: "text/plain",
        buffer: Buffer.from("not an anki file"),
    });
});

Then('отображается ошибка импорта Anki', async ({ page }) => {
    await expect(page.getByTestId(/anki.*error|error.*anki/)).toBeVisible({ timeout: 15_000 });
});

When('загружает валидный Anki файл', async ({ page }) => {
    const wordsPage = new WordsPage(page);
    await wordsPage.uploadAnkiFile("fixtures/sample.apkg");
    await page.waitForTimeout(3000);
});

When('переключает на вкладку изображения', async ({ page }) => {
    const wordsPage = new WordsPage(page);
    await wordsPage.switchToImageTab();
});

When('загружает изображение для распознавания', async ({ page }) => {
    const wordsPage = new WordsPage(page);
    await wordsPage.uploadImageFile(path.resolve(__dirname, "../../../origa/src/ocr/ocr_example.jpg"));
    await page.waitForTimeout(5000);
});

When('переключает на вкладку аудио', async ({ page }) => {
    const wordsPage = new WordsPage(page);
    await wordsPage.switchToAudioTab();
});

When('загружает аудио для транскрипции', async ({ page }) => {
    const wordsPage = new WordsPage(page);
    await wordsPage.uploadAudioFile(path.resolve(__dirname, "../../fixtures/standard_sample1.wav"));
    await page.waitForTimeout(5000);
});
