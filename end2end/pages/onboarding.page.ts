import { Page, Locator, expect } from "@playwright/test";
import { BasePage } from "./base.page";

export class OnboardingPage extends BasePage {
    // Page structure
    readonly onboardingPage: Locator;
    readonly onboardingCard: Locator;
    readonly onboardingContent: Locator;
    readonly onboardingStepper: Locator;

    // Loading
    readonly onboardingLoading: Locator;
    readonly onboardingSpinner: Locator;

    // Navigation buttons
    readonly prevButton: Locator;
    readonly nextButton: Locator;
    readonly importButton: Locator;

    // Steps
    readonly introStep: Locator;
    readonly loadStep: Locator;
    readonly jlptStep: Locator;
    readonly appsStep: Locator;
    readonly progressStep: Locator;
    readonly summaryStep: Locator;

    // Intro step
    readonly introTitle: Locator;
    readonly introSubtitle: Locator;

    // Intro step — language
    readonly introLanguageBar: Locator;
    readonly languageEnButton: Locator;
    readonly languageRuButton: Locator;

    // Intro step — feature showcase
    readonly introFeatureShowcase: Locator;
    readonly introFeatureVocabulary: Locator;
    readonly introFeatureKanji: Locator;
    readonly introFeatureGrammar: Locator;
    // Intro step — feature tags
    readonly introTagVocabulary: Locator;
    readonly introTagKanji: Locator;
    readonly introTagGrammar: Locator;

    // JLPT step
    readonly jlptTitle: Locator;
    readonly jlptSubtitle: Locator;

    // Apps step
    readonly appsTitle: Locator;
    readonly appsSubtitle: Locator;

    // Summary step
    readonly summaryTitle: Locator;    readonly summaryStats: Locator;

    readonly summarySubtitle: Locator;

    // Scoring step
    readonly scoringStep: Locator;
    readonly scoringHint: Locator;
    readonly scoringProgress: Locator;
    readonly scoringQuestion: Locator;
    readonly scoringAnswer: Locator;
    readonly scoringDontKnowBtn: Locator;
    readonly scoringKnowBtn: Locator;
    readonly scoringComplete: Locator;
    readonly markAllKnownBtn: Locator;
    readonly skipScoringBtn: Locator;
    readonly finishButton: Locator;

    constructor(page: Page) {
        super(page);

        // Page structure
        this.onboardingPage = page.getByTestId("onboarding-page");
        this.onboardingCard = page.getByTestId("onboarding-card");
        this.onboardingContent = page.getByTestId("onboarding-content");
        this.onboardingStepper = page.getByTestId("onboarding-stepper");

        // Loading
        this.onboardingLoading = page.getByTestId("onboarding-loading");
        this.onboardingSpinner = page.getByTestId("onboarding-spinner");

        // Navigation buttons
        this.prevButton = page.getByTestId("onboarding-prev");
        this.nextButton = page.getByTestId("onboarding-next");
        this.importButton = page.getByTestId("onboarding-import");

        // Steps (actual test IDs from mod.rs)
        this.introStep = page.getByTestId("onboarding-intro-step");
        this.loadStep = page.getByTestId("onboarding-load-step");
        this.jlptStep = page.getByTestId("onboarding-jlpt-step");
        this.appsStep = page.getByTestId("onboarding-apps-step");
        this.progressStep = page.getByTestId("onboarding-progress-step");
        this.summaryStep = page.getByTestId("onboarding-summary-step");

        // Intro step
        this.introTitle = page.getByTestId("intro-step-title");
        this.introSubtitle = page.getByTestId("intro-step-subtitle");

        // Intro step — language
        this.introLanguageBar = page.getByTestId("intro-step-language-bar");
        this.languageEnButton = page.getByTestId("lang-toggle-en");
        this.languageRuButton = page.getByTestId("lang-toggle-ru");

        // Intro step — feature showcase
        this.introFeatureShowcase = page.getByTestId("intro-step-feature-showcase");
        this.introFeatureVocabulary = page.getByTestId("intro-step-feature-vocabulary");
        this.introFeatureKanji = page.getByTestId("intro-step-feature-kanji");
        this.introFeatureGrammar = page.getByTestId("intro-step-feature-grammar");
        // Intro step — feature tags
        this.introTagVocabulary = page.getByTestId("intro-step-tag-vocabulary");
        this.introTagKanji = page.getByTestId("intro-step-tag-kanji");
        this.introTagGrammar = page.getByTestId("intro-step-tag-grammar");

        // JLPT step
        this.jlptTitle = page.getByTestId("jlpt-title");
        this.jlptSubtitle = page.getByTestId("jlpt-subtitle");

        // Apps step
        this.appsTitle = page.getByTestId("apps-title");
        this.appsSubtitle = page.getByTestId("apps-subtitle");

        // Summary step
        this.summaryStats = page.getByTestId("summary-step-stats");
        this.summaryTitle = page.getByTestId("summary-title");
        this.summarySubtitle = page.getByTestId("summary-subtitle");

        // Scoring step
        this.scoringStep = page.getByTestId("onboarding-scoring-step");
        this.scoringHint = page.getByTestId("scoring-step-hint");
        this.scoringProgress = page.getByTestId("scoring-step-progress");
        this.scoringQuestion = page.getByTestId("scoring-step-question");
        this.scoringAnswer = page.getByTestId("scoring-step-answer");
        this.scoringDontKnowBtn = page.getByTestId("scoring-step-dont-know");
        this.scoringKnowBtn = page.getByTestId("scoring-step-know");
        this.scoringComplete = page.getByTestId("scoring-step-complete");
        this.markAllKnownBtn = page.getByTestId("onboarding-mark-all-known");
        this.skipScoringBtn = page.getByTestId("onboarding-skip-scoring");
        this.finishButton = page.getByTestId("onboarding-finish");
    }

    async goto(): Promise<void> {
        await this.navigate("/onboarding");
    }

    async expectOnboardingVisible(): Promise<void> {
        await expect(this.onboardingPage).toBeVisible();
        await expect(this.onboardingCard).toBeVisible();
    }

    async selectJlptLevel(level: "N5" | "N4" | "N3" | "N2" | "N1" | "unknown"): Promise<void> {
        await this.page.getByText(level, { exact: false }).first().click();
    }

    async toggleApp(appId: string): Promise<void> {
        const appCheckbox = this.page.getByTestId(`apps-step-app-${appId}-checkbox`);
        await appCheckbox.click();
    }

    async toggleSet(setId: string): Promise<void> {
        const setCheckbox = this.page.getByTestId(`summary-step-set-${setId}-checkbox`);
        await setCheckbox.click();
    }

    /** Locator for a summary set checkbox — for checked-state assertions. */
    setCheckboxLocator(setId: string): Locator {
        return this.page.getByTestId(`summary-step-set-${setId}-checkbox`);
    }

    async isAppSelected(appId: string): Promise<boolean> {
        const appCard = this.page.getByTestId(`apps-step-app-${appId}`);
        const classAttribute = await appCard.getAttribute("class");
        return classAttribute?.includes("selected") ?? false;
    }

    async getSelectedSetsCount(): Promise<string> {
        const statsText = await this.page.getByTestId("summary-step-stats").textContent();
        return statsText ?? "";
    }

    async goToNextStep(): Promise<void> {
        await this.nextButton.click();
    }

    async goToPrevStep(): Promise<void> {
        await this.prevButton.click();
    }

    async startImport(): Promise<void> {
        await this.importButton.click();
    }

    async expectStepVisible(stepTestId: string): Promise<void> {
        await expect(this.page.getByTestId(stepTestId)).toBeVisible();
    }

    async selectMinnaLevel(level: "N5" | "N4"): Promise<void> {
        await this.page.getByTestId("minna-level-dropdown").click();
        await this.page.getByTestId(`minna-level-dropdown-option-${level}`).click();
    }

    async selectMinnaLesson(lessonNumber: number): Promise<void> {
        await this.page.getByTestId("minna-lesson-dropdown").click();
        await this.page.getByTestId(`minna-lesson-dropdown-option-lesson_${lessonNumber}`).click();
    }

    async selectMigiiLevel(level: string): Promise<void> {
        await this.page.getByTestId("migii-level-dropdown").click();
        await this.page.getByTestId(`migii-level-dropdown-option-${level}`).click();
    }

    async selectMigiiLesson(lessonNumber: number): Promise<void> {
        await this.page.getByTestId("migii-lesson-dropdown").click();
        await this.page.getByTestId(`migii-lesson-dropdown-option-lesson_${lessonNumber}`).click();
    }

    async skipOnboarding(): Promise<void> {
        await this.page.getByTestId("onboarding-skip").click();
        await this.page.getByTestId("onboarding-confirm-ok").click();
    }

    async clickKnow(): Promise<void> {
        await this.scoringKnowBtn.click();
    }

    async clickDontKnow(): Promise<void> {
        await this.scoringDontKnowBtn.click();
    }

    async clickMarkAllKnown(): Promise<void> {
        await this.markAllKnownBtn.click();
        await this.page.getByTestId("onboarding-confirm-ok").click();
    }

    async clickSkipScoring(): Promise<void> {
        await this.skipScoringBtn.click();
        await this.page.getByTestId("onboarding-confirm-ok").click();
    }

    async clickFinish(): Promise<void> {
        await this.finishButton.click();
    }

    async getScoringProgress(): Promise<string> {
        return (await this.scoringProgress.textContent()) ?? "";
    }
}
