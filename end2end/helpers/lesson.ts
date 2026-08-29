import { expect, type Page } from "@playwright/test";
import { HomePage, LessonPage, WordsPage } from "../pages";
import { skipOnboarding } from "./navigation";

export const MAX_LESSON_ITERATIONS = 50;
// Bounded timeout for individual card actions. Without it a click on a
// card-control element races with WASM re-renders: Playwright keeps
// retrying the DETACHED element handle until the whole test times out
// ("element was detached from the DOM, retrying" forever). A bounded
// action timeout turns that hang into a fast failure, and the loop's
// next iteration re-resolves the locators against the fresh DOM.
// LessonPage re-exports this as CARD_ACTION_TIMEOUT for its methods.
export const ACTION_TIMEOUT = 10_000;

/// Рука знакомства: проходит показ кнопкой «Дальше» до тренировки.
export async function runAcquaintancePresentation(page: Page): Promise<void> {
	const nextBtn = page.getByTestId("acquaintance-next-btn");
	for (let i = 0; i < 20; i++) {
		if (!(await nextBtn.isVisible().catch(() => false))) break;
		await nextBtn.click();
		if (
			await page
				.getByTestId("acquaintance-training")
				.isVisible()
				.catch(() => false)
		)
			break;
	}
}

/// Проходит руку знакомства целиком (показ → тренировка до критерия),
/// если она показана. Возвращает true, когда рука была.
export async function completeAcquaintanceHandIfPresent(
	page: Page,
): Promise<boolean> {
	const view = page.getByTestId("acquaintance-view");
	if (!(await view.isVisible({ timeout: 10_000 }).catch(() => false))) {
		return false;
	}
	await runAcquaintancePresentation(page);
	for (let i = 0; i < 100; i++) {
		const reveal = page.getByTestId("acquaintance-reveal-btn");
		if (!(await reveal.isVisible().catch(() => false))) break;
		await reveal.click();
		await page.getByTestId("acquaintance-rating-remember").click();
	}
	await expect(view).not.toBeVisible({ timeout: 15_000 });
	return true;
}

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

        // Pure-manual advance (ADR-033): after submitting a quiz/yesno
        // answer the user is held on the feedback card until they click
        // "Next" (or press Space/Enter). The check MUST run before the
        // `anyInteractive` wait below: on VOCABULARY quiz cards the options
        // are hidden once the result is shown (quiz_card.rs renders either
        // QuizOptions or QuizResultDisplay, never both), so during the
        // feedback phase NO interactive element from `anyInteractive` is
        // visible and the wait would time out. Phrase quizzes keep their
        // options visible — hence the separate check (never both in one
        // `.or()`, which would trip Playwright strict mode; see below).
        if (await lessonPage.lessonCardNextBtn.isVisible().catch(() => false)) {
            await lessonPage.clickNextCard().catch(() => {});
            continue;
        }

        // `anyInteractive` deliberately excludes `lessonCardNextBtn`: after
        // submitting a phrase quiz both the (still-visible) quiz options
        // and the freshly-shown NextCardButton are in the DOM, so including
        // both in the same `.or()` chain would trip Playwright strict mode.
        const anyInteractive = lessonPage.showAnswerBtn
            .or(lessonPage.quizOptions[0])
            .or(lessonPage.yesnoYesBtn)
            .or(lessonPage.completeScreen);
        await expect(anyInteractive).toBeVisible({ timeout: 15_000 });

        if (await lessonPage.completeScreen.isVisible().catch(() => false)) break;

        if (await lessonPage.showAnswerBtn.isVisible().catch(() => false)) {
            await lessonPage.showAnswer();
            await lessonPage.rate("good");
        } else if (await lessonPage.quizOptions[0].isVisible().catch(() => false)) {
            await lessonPage.selectQuizOption(0);
        } else if (await lessonPage.yesnoYesBtn.isVisible().catch(() => false)) {
            await lessonPage.yesnoYesBtn.click({ timeout: ACTION_TIMEOUT });
        } else {
            break;
        }    }
}
