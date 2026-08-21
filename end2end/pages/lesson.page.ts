import { Page, Locator, expect } from "@playwright/test";
import { BasePage } from "./base.page";

type Rating = "again" | "good";

export class LessonPage extends BasePage {
    // Page structure
    readonly lessonPage: Locator;
    readonly lessonCard: Locator;
    readonly lessonHeader: Locator;

    // Navigation
    readonly backButton: Locator;
    readonly muteButton: Locator;

    // Loading states
    readonly lessonLoading: Locator;
    readonly lessonSpinner: Locator;
    readonly lessonLoadingText: Locator;

    // Error states
    readonly lessonError: Locator;

    // Empty state (diagnosed deck exhaustion / daily limit / next review)
    readonly lessonEmptyState: Locator;
    readonly lessonEmptyImportBtn: Locator;
    readonly lessonEmptyProfileBtn: Locator;
    readonly lessonEmptyNextReview: Locator;


    // Content
    readonly lessonContent: Locator;

    // Card interaction
    readonly showAnswerBtn: Locator;
    readonly ratingAgain: Locator;
    readonly ratingGood: Locator;
    readonly progressText: Locator;

    // Quiz
    readonly quizOptions: readonly [Locator, Locator, Locator, Locator];

    // Yes/No
    readonly yesnoYesBtn: Locator;
    readonly yesnoNoBtn: Locator;

    // Pure-manual advance (ADR-033): in-lesson "Next card" button shown after
    // a quiz/yesno/phrase answer is submitted. Distinct from `nextLessonBtn`
    // which is on the completion screen and starts the NEXT lesson.
    readonly lessonCardNextBtn: Locator;

    // Lesson completion
    readonly completeScreen: Locator;
    readonly completeBackBtn: Locator;
    readonly completeStats: Locator;
    readonly nextLessonBtn: Locator;
    readonly homeBtn: Locator;

    // Sync indicator
    readonly syncIndicator: Locator;
    readonly syncSpinner: Locator;

    private readonly ratingMap: Record<Rating, Locator>;

    constructor(page: Page) {
        super(page);

        // Page structure
        this.lessonPage = page.getByTestId("lesson-page");
        this.lessonCard = page.getByTestId("lesson-card");
        this.lessonHeader = page.getByTestId("lesson-header");

        // Navigation
        this.backButton = page.getByTestId("lesson-back-btn");
        this.muteButton = page.getByTestId("lesson-mute-btn");

        // Loading states
        this.lessonLoading = page.getByTestId("lesson-loading");
        this.lessonSpinner = page.getByTestId("lesson-spinner");
        this.lessonLoadingText = page.getByTestId("lesson-loading-text");

        // Error states
        this.lessonError = page.getByTestId("lesson-error");

        // Empty state
        this.lessonEmptyState = page.getByTestId("lesson-empty-state");
        this.lessonEmptyImportBtn = page.getByTestId("lesson-empty-import-btn");
        this.lessonEmptyProfileBtn = page.getByTestId("lesson-empty-profile-btn");
        this.lessonEmptyNextReview = page.getByTestId("lesson-empty-next-review");


        // Content
        this.lessonContent = page.getByTestId("lesson-content");

        // Card interaction
        this.showAnswerBtn = page.getByTestId("lesson-show-answer-btn");
        this.ratingAgain = page.getByTestId("lesson-rating-btn-again");
        this.ratingGood = page.getByTestId("lesson-rating-btn-good");
        this.progressText = page.getByTestId("lesson-progress-text");

        // Quiz
        this.quizOptions = [
            page.getByTestId("quiz-option-0"),
            page.getByTestId("quiz-option-1"),
            page.getByTestId("quiz-option-2"),
            page.getByTestId("quiz-option-3"),
        ] as const;

        // Yes/No
        this.yesnoYesBtn = page.getByTestId("yesno-yes-btn");
        this.yesnoNoBtn = page.getByTestId("yesno-no-btn");

        // Pure-manual advance (ADR-033): in-lesson "Next card" button.
        this.lessonCardNextBtn = page.getByTestId("lesson-card-next-btn");

        // Lesson completion
        this.completeScreen = page.getByTestId("lesson-complete-screen");
        this.completeBackBtn = page.getByTestId("lesson-complete-back-btn");
        this.completeStats = page.getByTestId("lesson-complete-stats");
        this.nextLessonBtn = page.getByTestId("lesson-next-btn");
        this.homeBtn = page.getByTestId("lesson-home-btn");

        // Sync indicator
        this.syncIndicator = page.getByTestId("lesson-sync-indicator");
        this.syncSpinner = page.getByTestId("lesson-sync-spinner");

        this.ratingMap = {
            again: this.ratingAgain,
            good: this.ratingGood,
        };
    }

    async goto(): Promise<void> {
        await this.navigate("/lesson");
    }

    async expectLessonVisible(): Promise<void> {
        await expect(this.lessonPage).toBeVisible();
        await expect(this.lessonCard).toBeVisible();
        await expect(this.lessonHeader).toBeVisible();
    }

    async waitForLoading(): Promise<void> {
        await expect(this.lessonLoading).toBeVisible({ timeout: 10_000 });
        await expect(this.lessonLoading).toBeHidden({ timeout: 30_000 });
    }

    async clickBack(): Promise<void> {
        await this.backButton.click();
    }

    async toggleMute(): Promise<void> {
        await this.muteButton.click();
    }

    async expectContentVisible(): Promise<void> {
        await expect(this.lessonContent).toBeVisible();
    }

    async expectErrorVisible(): Promise<void> {
        await expect(this.lessonError).toBeVisible();
    }

    async expectEmptyStateVisible(): Promise<void> {
        await expect(this.lessonEmptyState).toBeVisible();
        await expect(this.lessonContent).toBeHidden();
    }

    async clickEmptyImportSets(): Promise<void> {
        await this.lessonEmptyImportBtn.click();
    }

    async clickEmptyIncreaseLoad(): Promise<void> {
        await this.lessonEmptyProfileBtn.click();
    }

    /**
     * Advance to the next card under the pure-manual advance model
     * (ADR-033). After a quiz/yesno/phrase answer is submitted, the user is
     * held on the feedback card until they press Space/Enter or click the
     * "Next" button. The click is bounded (10s): under WASM re-render races
     * the resolved element can be detached mid-click, and without a timeout
     * Playwright retries the stale handle until the TEST timeout; callers
     * re-resolve locators on their next loop iteration.
     */
    async clickNextCard(): Promise<void> {
        await this.lessonCardNextBtn.click({ timeout: 10_000 });
    }

    async waitForSync(): Promise<void> {
        await expect(this.syncIndicator).toBeVisible({ timeout: 10_000 });
        await expect(this.syncIndicator).toBeHidden({ timeout: 30_000 });
    }

    async showAnswer(): Promise<void> {
        await this.showAnswerBtn.click();
    }

    async rate(rating: Rating): Promise<void> {
        await this.ratingMap[rating].click();
    }

    async getProgressText(): Promise<string> {
        const text = await this.progressText.textContent();
        return text ?? "";
    }

    async waitForComplete(): Promise<void> {
        await expect(this.completeScreen).toBeVisible({ timeout: 30_000 });
    }

    async clickNextLesson(): Promise<void> {
        await this.nextLessonBtn.click();
    }

    async clickHome(): Promise<void> {
        await this.homeBtn.click();
    }

    async expectMuteIcon(): Promise<void> {
        await expect(this.muteButton.locator("svg")).toBeVisible();
    }

    async selectQuizOption(index: number): Promise<void> {
        await this.quizOptions[index].click();
    }
}