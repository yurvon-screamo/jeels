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

When('вводит нераспознаваемый текст для анализа', async ({ page }) => {
    const wordsPage = new WordsPage(page);
    await wordsPage.openAddModal();
    // Latin punctuation only — lindera tokenizes Japanese; this yields 0
    // AnalyzedWords and triggers the NoResults feedback branch.
    await wordsPage.enterText(",,, ... !!!");
    await wordsPage.analyzeTextNoResults();
});

Then('отображается сообщение об отсутствии найденных слов', async ({ page }) => {
    const wordsPage = new WordsPage(page);
    await expect(wordsPage.noResultsFeedback).toBeVisible();
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
    await expect(wordsPage.emptyState).toBeVisible({ timeout: 10_000 });
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
    await wordsPage.uploadAnkiFile(path.resolve(__dirname, "../../fixtures/sample.apkg"));

    // Anki import is a multi-stage flow inside one user action:
    //   1. .apkg upload → field-mapping stage
    //   2. user picks the "Expression" column as the word source
    //   3. preview of cards to import
    //   4. confirm import → drawer closes, cards land on the words page
    await expect(wordsPage.ankiFieldWord).toBeVisible({ timeout: 30_000 });
    await wordsPage.ankiFieldWord.click();
    await page.getByTestId("anki-import-field-word-option-Expression").click();

    await wordsPage.ankiNextBtn.click();
    await expect(wordsPage.ankiCardCount).toBeVisible({ timeout: 30_000 });

    await wordsPage.ankiImportBtn.click();
    await expect(wordsPage.drawer).not.toBeVisible({ timeout: 30_000 });
});

When('переключает на вкладку изображения', async ({ page }) => {
    const wordsPage = new WordsPage(page);
    await wordsPage.switchToImageTab();
});

When('загружает изображение для распознавания', async ({ page }) => {
    const wordsPage = new WordsPage(page);
    await wordsPage.uploadImageFile(path.resolve(__dirname, "../../../origa/src/ocr/ocr_example.jpg"));
    // OCR pulls ML models on first run (~50 MB), then recognizes text and
    // auto-analyzes it. The "Найдено" marker indicates analysis is complete.
    await wordsPage.drawer.getByText(/Найдено/).waitFor({ state: "visible", timeout: 600_000 });
    // Pick the first recognized word and add it.
    await wordsPage.selectFirstWord();
    await wordsPage.addSelectedWords();
});

When('переключает на вкладку аудио', async ({ page }) => {
    const wordsPage = new WordsPage(page);
    await wordsPage.switchToAudioTab();
});

When('загружает аудио для транскрипции', async ({ page }) => {
    const wordsPage = new WordsPage(page);
    await wordsPage.uploadAudioFile(path.resolve(__dirname, "../../fixtures/standard_sample1.wav"));
    // Whisper model is pulled on first run (~75 MB), then audio is transcribed
    // and the resulting text is auto-analyzed.
    await wordsPage.drawer.getByText(/Найдено/).waitFor({ state: "visible", timeout: 600_000 });
    await wordsPage.selectFirstWord();
    await wordsPage.addSelectedWords();
});
