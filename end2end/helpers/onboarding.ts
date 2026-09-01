import { type Page, expect } from "@playwright/test";

/**
 * Waits for the scoring step to finish loading.
 * Resolves when either scoring-step-hint (cards ready) or
 * scoring-step-complete (0 new cards) is visible.
 * Throws if neither appears within timeout.
 */
export async function waitForScoringReady(page: Page, timeout = 30_000): Promise<void> {
    await Promise.race([
        page.getByTestId("scoring-step-hint").waitFor({ state: "visible", timeout }),
        page.getByTestId("scoring-step-complete").waitFor({ state: "visible", timeout }),
    ]);
}

export interface CompleteOnboardingOptions {
    /**
     * Display name typed into the intro step's name input before moving on.
     * The value commits via the intro-save callback when the user leaves the
     * intro step (the Next button in onboarding/mod.rs).
     */
    displayName?: string;
}

/**
 * Completes onboarding from login through import, stops at scoring step.
 * Navigates: Intro → Load → JLPT (N4) → Apps → Progress → Summary → Import → Scoring
 *
 * Returns `true` if scoring step was reached and is ready, `false` if redirected to /home.
 *
 * NOTE: Extracted from onboarding.spec.ts. The local copy in the spec file
 * was removed; both spec and BDD steps import from here.
 */
export async function completeOnboardingToScoring(
    page: Page,
    options: CompleteOnboardingOptions = {},
): Promise<boolean> {
    await page.goto("/");

    try {
        await page.waitForURL(/\/onboarding$/, { timeout: 30_000 });
    } catch {
        if (page.url().includes("/home")) {
            return false;
        }
    }

    if (page.url().includes("/home")) {
        return false;
    }

    await expect(page.getByTestId("onboarding-spinner")).not.toBeVisible({ timeout: 10_000 });

    // Intro: optionally type a display name before leaving the step (the
    // name commits via the intro-save callback once the step is left).
    if (options.displayName !== undefined) {
        await page.getByTestId("intro-step-name-input").fill(options.displayName);
    }

    // Intro → Load
    await page.getByTestId("onboarding-next").click();
    await expect(page.getByTestId("onboarding-load-step")).toBeVisible();

    // Load → JLPT (default medium load)
    await page.getByTestId("onboarding-next").click();
    await expect(page.getByTestId("onboarding-jlpt-step")).toBeVisible();

    // JLPT: select N4
    await page.getByTestId("jlpt-option-n4").click();
    await expect(page.getByTestId("jlpt-option-n4")).toHaveClass(/selected/, { timeout: 5000 });
    await page.getByTestId("onboarding-next").click();
    await expect(page.getByTestId("onboarding-apps-step")).toBeVisible();

    // Apps: select Migii, DuolingoRu, MinnaNoNihongo, Irodori
    const migiiCheckbox = page.getByTestId("apps-step-app-Migii-checkbox");
    if (await migiiCheckbox.isVisible().catch(() => false)) {
        await migiiCheckbox.click();
    }

    const duolingoRuCheckbox = page.getByTestId("apps-step-app-DuolingoRu-checkbox");
    if (await duolingoRuCheckbox.isVisible().catch(() => false)) {
        await duolingoRuCheckbox.click();
    }

    const minnaCheckbox = page.getByTestId("apps-step-app-MinnaNoNihongo-checkbox");
    if (await minnaCheckbox.isVisible().catch(() => false)) {
        await minnaCheckbox.click();
    }

    const irodoriCheckbox = page.getByTestId("apps-step-app-Irodori-checkbox");
    if (await irodoriCheckbox.isVisible().catch(() => false)) {
        await irodoriCheckbox.click();
    }

    await page.getByTestId("onboarding-next").click();
    await expect(page.getByTestId("onboarding-progress-step")).toBeVisible();

    // Progress: configure each selected app
    const migiiLevelDropdown = page.getByTestId("migii-level-dropdown");
    if (await migiiLevelDropdown.isVisible().catch(() => false)) {
        await migiiLevelDropdown.click();
        await page.getByTestId("migii-level-dropdown-option-N4").click();

        const migiiLessonDropdown = page.getByTestId("migii-lesson-dropdown");
        await migiiLessonDropdown.click();
        await page.getByTestId("migii-lesson-dropdown-option-lesson_10").click();
    }

    const duolingoRuModuleDropdown = page.getByTestId("DuolingoRu-module-dropdown");
    if (await duolingoRuModuleDropdown.isVisible().catch(() => false)) {
        await duolingoRuModuleDropdown.click();
        await page.getByTestId("DuolingoRu-module-dropdown-option-module_1").click();

        const duolingoRuUnitDropdown = page.getByTestId("DuolingoRu-unit-dropdown");
        await duolingoRuUnitDropdown.click();
        await page.getByTestId("DuolingoRu-unit-dropdown-option-unit_10").click();
    }

    const minnaLevelDropdown = page.getByTestId("minna-level-dropdown");
    if (await minnaLevelDropdown.isVisible().catch(() => false)) {
        await minnaLevelDropdown.click();
        await page.getByTestId("minna-level-dropdown-option-N4").click();

        const minnaLessonDropdown = page.getByTestId("minna-lesson-dropdown");
        await minnaLessonDropdown.click();
        await page.getByTestId("minna-lesson-dropdown-option-lesson_38").click();
    }

    const irodoriBookDropdown = page.getByTestId("irodori-book-dropdown");
    if (await irodoriBookDropdown.isVisible().catch(() => false)) {
        await irodoriBookDropdown.click();
        await page.getByTestId("irodori-book-dropdown-option-nyuumon").click();

        const irodoriLessonDropdown = page.getByTestId("irodori-lesson-dropdown");
        await irodoriLessonDropdown.click();
        await page.getByTestId("irodori-lesson-dropdown-option-lesson_9").click();
    }

    await page.getByTestId("onboarding-next").click();
    await expect(page.getByTestId("onboarding-summary-step")).toBeVisible();

    // Summary → Import → Scoring
    await page.getByTestId("onboarding-import").click();
    await expect(page.getByTestId("onboarding-import")).toHaveAttribute("data-loading", "true", { timeout: 5000 });
    await expect(page.getByTestId("onboarding-scoring-step")).toBeVisible({ timeout: 120_000 });

    // Wait for scoring step to finish loading cards before returning
    await waitForScoringReady(page);

    return true;
}
