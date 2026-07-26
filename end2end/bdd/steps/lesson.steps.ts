import { expect } from "@playwright/test";
import { When, Then } from "../fixtures";
import { HomePage, LessonPage, WordsPage } from "../../pages";
import { completeLessonFlexible } from "../../helpers/lesson";

When('пользователь добавил слово из текста {string}', async ({ page }, text: string) => {
    const wordsPage = new WordsPage(page);
    await wordsPage.goto();
    await wordsPage.expectWordsVisible();
    await wordsPage.openAddModal();
    await wordsPage.enterText(text);
    await wordsPage.analyzeText();
    await wordsPage.selectFirstWord();
    await wordsPage.addSelectedWords();
    await expect(wordsPage.wordsGrid).toBeVisible({ timeout: 10_000 });
});

When('пользователь начинает урок', async ({ page }) => {
    const homePage = new HomePage(page);
    await homePage.goto();
    await homePage.startLesson();
});

Then('отображается страница урока с карточкой', async ({ page }) => {
    const lessonPage = new LessonPage(page);
    await expect(lessonPage.lessonPage).toBeVisible({ timeout: 15_000 });
    await expect(lessonPage.lessonError).not.toBeVisible({ timeout: 15_000 });
    await expect(lessonPage.lessonLoading).toBeHidden({ timeout: 30_000 });
    await expect(lessonPage.lessonContent).toBeVisible({ timeout: 15_000 });
});

When('оценивает каждую карточку как Good', async ({ page }) => {
    const lessonPage = new LessonPage(page);
    await completeLessonFlexible(lessonPage, page);
});

Then('отображается экран завершения урока', async ({ page }) => {
    const lessonPage = new LessonPage(page);
    await expect(lessonPage.completeScreen).toBeVisible({ timeout: 15_000 });
});

When('нажимает кнопку показа ответа', async ({ page }) => {
    const lessonPage = new LessonPage(page);
    await lessonPage.showAnswer();
});

Then('отображаются кнопки оценки', async ({ page }) => {
    const lessonPage = new LessonPage(page);
    await expect(lessonPage.ratingAgain).toBeVisible();
    await expect(lessonPage.ratingGood).toBeVisible();
});

When('нажимает кнопку возврата с урока', async ({ page }) => {
    const lessonPage = new LessonPage(page);
    await lessonPage.clickBack();
});

Then('отображается текст прогресса урока', async ({ page }) => {
    const lessonPage = new LessonPage(page);
    await expect(lessonPage.progressText).toBeVisible();
});

When('нажимает кнопку звука', async ({ page }) => {
    const lessonPage = new LessonPage(page);
    await lessonPage.toggleMute();
});

Then('звук переключён', async ({ page }) => {
    const lessonPage = new LessonPage(page);
    await expect(lessonPage.muteButton).toHaveAttribute("data-muted", "true");
});

Then('отображается статистика завершения', async ({ page }) => {
    const lessonPage = new LessonPage(page);
    await expect(lessonPage.completeStats).toBeVisible();
});
