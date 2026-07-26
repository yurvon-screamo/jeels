import { expect, type Page } from "@playwright/test";
import { HomePage, LessonPage, WordsPage } from "../pages";
import { skipOnboarding } from "./navigation";

export const MAX_LESSON_ITERATIONS = 50;

export async function setupLessonWithCards(page: Page): Promise<LessonPage> {
    await skipOnboarding(page);

    const wordsPage = new WordsPage(page);
    await wordsPage.goto();
    await wordsPage.expectWordsVisible();
    await wordsPage.openAddModal();
    await wordsPage.enterText("私は本を読みます");
    await wordsPage.analyzeText();
    await wordsPage.selectFirstWord();
    await wordsPage.addSelectedWords();
    await expect(wordsPage.wordsGrid).toBeVisible({ timeout: 10_000 });

    const homePage = new HomePage(page);
    await homePage.goto();
    await homePage.startLesson();

    const lessonPage = new LessonPage(page);

    await expect(lessonPage.lessonPage).toBeVisible({ timeout: 15_000 });
    await expect(lessonPage.lessonError).not.toBeVisible({ timeout: 15_000 });
    await expect(lessonPage.lessonLoading).toBeHidden({ timeout: 30_000 });
    await expect(lessonPage.lessonContent).toBeVisible({ timeout: 15_000 });
    await expect(lessonPage.showAnswerBtn).toBeVisible({ timeout: 15_000 });

    return lessonPage;
}

export async function rateCardUntilDone(
    lessonPage: LessonPage,
    rating: "again" | "good",
    maxIterations = MAX_LESSON_ITERATIONS,
): Promise<void> {
    for (let i = 0; i < maxIterations; i++) {
        const isComplete = await lessonPage.completeScreen.isVisible().catch(() => false);
        if (isComplete) break;
        try {
            await lessonPage.showAnswer();
            await lessonPage.rate(rating);
            await expect(lessonPage.showAnswerBtn.or(lessonPage.completeScreen)).toBeVisible({
                timeout: 5000,
            });
        } catch {
            break;
        }
    }
}

export async function completeLessonFlexible(
    lessonPage: LessonPage,
    page: Page,
    maxIterations = MAX_LESSON_ITERATIONS,
): Promise<void> {
    for (let i = 0; i < maxIterations; i++) {
        const isComplete = await lessonPage.completeScreen.isVisible().catch(() => false);
        if (isComplete) break;

        // `anyInteractive` deliberately excludes `lessonCardNextBtn`: after
        // submitting a quiz/yesno answer, both the (still-visible) quiz
        // options and the freshly-shown NextCardButton are in the DOM, so
        // including both in the same `.or()` chain would trip Playwright
        // strict mode. The NextCardButton is checked separately below.
        const anyInteractive = lessonPage.showAnswerBtn
            .or(lessonPage.quizOptions[0])
            .or(lessonPage.yesnoYesBtn)
            .or(lessonPage.completeScreen);
        await expect(anyInteractive).toBeVisible({ timeout: 15_000 });

        if (await lessonPage.completeScreen.isVisible().catch(() => false)) break;

        // Pure-manual advance (ADR-033): after submitting a quiz/yesno
        // answer the user is held on the feedback card until they click
        // "Next" (or press Space/Enter). The previous 1500ms auto-advance
        // timer was removed — the helper must explicitly advance.
        if (await lessonPage.lessonCardNextBtn.isVisible().catch(() => false)) {
            await lessonPage.clickNextCard();
            continue;
        }

        if (await lessonPage.showAnswerBtn.isVisible().catch(() => false)) {
            await lessonPage.showAnswer();
            await lessonPage.rate("good");
        } else if (await lessonPage.quizOptions[0].isVisible().catch(() => false)) {
            await lessonPage.selectQuizOption(0);
        } else if (await lessonPage.yesnoYesBtn.isVisible().catch(() => false)) {
            await lessonPage.yesnoYesBtn.click();
        } else {
            break;
        }
    }
}
