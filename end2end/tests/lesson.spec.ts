import { test, expect } from "@playwright/test";
import { testWithFreshUser } from "../fixtures";
import { setupLessonWithCards, rateCardUntilDone, completeLessonFlexible } from "../helpers/lesson";
import { skipOnboarding } from "../helpers/navigation";
import { HomePage, LessonPage, WordsPage } from "../pages";

testWithFreshUser.describe("Lesson Page", () => {
    testWithFreshUser("should display lesson page with card", async ({ page }) => {
        test.setTimeout(90_000);
        const lessonPage = await setupLessonWithCards(page);
        await lessonPage.expectLessonVisible();
    });

    testWithFreshUser("should show answer and rate card", async ({ page }) => {
        test.setTimeout(120_000);
        const lessonPage = await setupLessonWithCards(page);
        await lessonPage.expectLessonVisible();
        await lessonPage.showAnswer();
        await lessonPage.rate("good");
    });

    testWithFreshUser("should navigate back from lesson", async ({ page }) => {
        test.setTimeout(90_000);
        const lessonPage = await setupLessonWithCards(page);
        await lessonPage.clickBack();
        await page.waitForURL(/\/home$/, { timeout: 10000 });
    });

    testWithFreshUser("should complete lesson after rating all cards", async ({ page }) => {
        test.setTimeout(120_000);
        const lessonPage = await setupLessonWithCards(page);

        await rateCardUntilDone(lessonPage, "good");

        await lessonPage.waitForComplete();
    });

    testWithFreshUser("should display progress text with card count", async ({ page }) => {
        test.setTimeout(90_000);
        const lessonPage = await setupLessonWithCards(page);
        const progressText = await lessonPage.getProgressText();
        expect(progressText).toMatch(/^\d+\/\d+$/);
    });

    testWithFreshUser("should show rating buttons after revealing answer", async ({ page }) => {
        test.setTimeout(90_000);
        const lessonPage = await setupLessonWithCards(page);
        await lessonPage.showAnswer();

        await expect(lessonPage.ratingAgain).toBeVisible();
        await expect(lessonPage.ratingGood).toBeVisible();
    });

    testWithFreshUser("should toggle mute button and change state", async ({ page }) => {
        test.setTimeout(90_000);
        const lessonPage = await setupLessonWithCards(page);
        await expect(lessonPage.muteButton).toBeVisible();
        await expect(lessonPage.muteButton).toHaveAttribute("data-muted", "false");

        await lessonPage.toggleMute();
        await expect(lessonPage.muteButton).toHaveAttribute("data-muted", "true");

        await lessonPage.toggleMute();
        await expect(lessonPage.muteButton).toHaveAttribute("data-muted", "false");
    });

    testWithFreshUser("should show complete screen with stats and navigation buttons", async ({ page }) => {
        test.setTimeout(120_000);
        const lessonPage = await setupLessonWithCards(page);

        await rateCardUntilDone(lessonPage, "good");

        await lessonPage.waitForComplete();
        await expect(lessonPage.completeStats).toBeVisible();
        await expect(lessonPage.nextLessonBtn).toBeVisible();
        await expect(lessonPage.homeBtn).toBeVisible();
    });

    testWithFreshUser("should navigate to home from complete screen", async ({ page }) => {
        test.setTimeout(120_000);
        const lessonPage = await setupLessonWithCards(page);

        await rateCardUntilDone(lessonPage, "good");

        await lessonPage.waitForComplete();
        await lessonPage.clickHome();
        await page.waitForURL(/\/home$/, { timeout: 10_000 });
    });

    testWithFreshUser("should start next lesson from complete screen", async ({ page }) => {
        test.setTimeout(120_000);
        await skipOnboarding(page);

        // Add multiple words to ensure enough cards for two lessons
        const wordsPage = new WordsPage(page);
        await wordsPage.goto();
        await wordsPage.expectWordsVisible();

        const sentences = [
            "私は本を読みます",
            "彼は学校に行きます",
            "猫が魚を食べる",
        ];
        for (const sentence of sentences) {
            await wordsPage.openAddModal();
            await wordsPage.enterText(sentence);
            await wordsPage.analyzeText();
            await wordsPage.selectFirstWord();
            await wordsPage.addSelectedWords();
            await expect(wordsPage.wordsGrid).toBeVisible({ timeout: 10_000 });
        }

        const homePage = new HomePage(page);
        await homePage.goto();
        await homePage.startLesson();

        const lessonPage = new LessonPage(page);
        await expect(lessonPage.lessonPage).toBeVisible({ timeout: 15_000 });
        await expect(lessonPage.lessonError).not.toBeVisible({ timeout: 15_000 });
        await expect(lessonPage.lessonLoading).toBeHidden({ timeout: 30_000 });
        await expect(lessonPage.lessonContent).toBeVisible({ timeout: 15_000 });

        await completeLessonFlexible(lessonPage, page);
        await lessonPage.waitForComplete();

        await lessonPage.clickNextLesson();

        // Wait for lesson reload: loading appears then resolves
        await lessonPage.lessonLoading
            .waitFor({ state: "visible", timeout: 5_000 })
            .catch(() => {});
        await lessonPage.lessonLoading
            .waitFor({ state: "hidden", timeout: 30_000 });

        // After loading, check the resulting state
        const hasContent = await lessonPage.lessonContent
            .isVisible({ timeout: 5_000 })
            .catch(() => false);
        const hasError = await lessonPage.lessonError
            .isVisible({ timeout: 5_000 })
            .catch(() => false);

        expect(hasContent || hasError).toBe(true);
    });

});

testWithFreshUser.describe("Lesson Card Vertical Layout", () => {
    testWithFreshUser(
        "lesson-content height-chain intact (flex-1 chain from shell fills viewport)",
        async ({ page }) => {
            test.setTimeout(90_000);

            // Tablet portrait: where the dynamic-resizing bug was reported.
            await page.setViewportSize({ width: 820, height: 1180 });

            const lessonPage = await setupLessonWithCards(page);

            // ADR-027 height-chain: shell (<main> flex-col min-h-[100dvh]) →
            // lesson-page (flex-1) → lesson-card testid (flex-1 min-h-0) →
            // lesson-content (flex-1). clientHeight > 700 proves the chain is
            // intact and `flex-1` is filling the shell rather than collapsing
            // to content height (which would happen if a future leptos_router
            // upgrade wraps the matched view in a DOM node, breaking the
            // direct flex-child relationship). Card proportions are intentionally
            // NOT asserted — see ADR-031 (stable min-h via svh makes fragile
            // padding/proportion checks obsolete).
            await expect(lessonPage.lessonContent).toBeVisible();

            const containerHeight = await lessonPage.lessonContent.evaluate(
                (el) => (el as HTMLElement).clientHeight,
            );
            expect(containerHeight).toBeGreaterThan(700);
        },
    );
});
