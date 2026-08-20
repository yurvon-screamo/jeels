import { expect } from "@playwright/test";
import { When, Then } from "../fixtures";
import { LessonPage, WordsPage } from "../../pages";

/**
 * Steps for the diagnosed lesson empty state (deck exhaustion, daily
 * new-card limit, next-review forecast). See lesson_empty_state.feature.
 */

Then('отображается пустое состояние урока', async ({ page }) => {
    const lessonPage = new LessonPage(page);
    await expect(lessonPage.lessonLoading).toBeHidden({ timeout: 30_000 });
    await lessonPage.expectEmptyStateVisible();
});

Then('отображается кнопка импорта наборов', async ({ page }) => {
    const lessonPage = new LessonPage(page);
    await expect(lessonPage.lessonEmptyImportBtn).toBeVisible();
});

Then('отображается кнопка увеличения нагрузки', async ({ page }) => {
    const lessonPage = new LessonPage(page);
    await expect(lessonPage.lessonEmptyProfileBtn).toBeVisible();
});

Then('отображается информация о ближайшем повторении', async ({ page }) => {
    const lessonPage = new LessonPage(page);
    await expect(lessonPage.lessonEmptyNextReview).toBeVisible();
});

When('нажимает кнопку импорта наборов из пустого урока', async ({ page }) => {
    const lessonPage = new LessonPage(page);
    await lessonPage.clickEmptyImportSets();
});

When('нажимает кнопку увеличения нагрузки из пустого урока', async ({ page }) => {
    const lessonPage = new LessonPage(page);
    await lessonPage.clickEmptyIncreaseLoad();
});

When('пользователь добавил слова из текста {string}', async ({ page }, text: string) => {
    const wordsPage = new WordsPage(page);
    await wordsPage.goto();
    await wordsPage.expectWordsVisible();
    await wordsPage.openAddModal();
    await wordsPage.enterText(text);
    await wordsPage.analyzeText();
    // The analyzer pre-selects every detected word — keep them all.
    await wordsPage.addSelectedWords();
    await expect(wordsPage.wordsGrid).toBeVisible({ timeout: 10_000 });
});
