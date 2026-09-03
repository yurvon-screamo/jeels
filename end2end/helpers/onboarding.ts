import { type Page, expect } from "@playwright/test";

import { OnboardingPage } from "../pages";

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

    /**
     * JLPT level picked at the level step. The summary then offers the
     * cumulative set (N5..level). Defaults to "N4" — the level every
     * existing caller was hardcoded to.
     */
    level?: "N5" | "N4" | "N3" | "N2" | "N1";

    /**
     * Wait for the scoring step to become ready after the import. The
     * default 30s is plenty for N4; a full N1 corpus import (~8k words,
     * ~11k+ cards) needs minutes in WASM.
     */
    scoringReadyTimeout?: number;

    /**
     * Skip app selection at the apps step. External-app progress EXCLUDES
     * already-studied sets from the import, so a full-corpus seed must not
     * select any app.
     */
    skipApps?: boolean;

    /**
     * Budget for the login → /onboarding redirect. The default 30s is fine
     * for release builds; a cold DEBUG wasm build (local `trunk serve`)
     * can take well over a minute on first load.
     */
    onboardingUrlTimeout?: number;

    /**
     * Stop at the summary step instead of running the import — import
     * threshold assertions read the summary BEFORE the corpus lands.
     * The boolean return value is meaningless in this mode (scoring is
     * never reached); the caller continues via the page object.
     */
    stopAtSummary?: boolean;
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
        await page.waitForURL(/\/onboarding$/, {
            timeout: options.onboardingUrlTimeout ?? 30_000,
        });
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

    // JLPT: select the requested level (cumulative corpus N5..level).
    const level = options.level ?? "N4";
    await page.getByTestId(`jlpt-option-${level.toLowerCase()}`).click();
    await expect(page.getByTestId(`jlpt-option-${level.toLowerCase()}`)).toHaveClass(/selected/, {
        timeout: 5000,
    });
    await page.getByTestId("onboarding-next").click();
    await expect(page.getByTestId("onboarding-apps-step")).toBeVisible();

    // Apps: select Migii, DuolingoRu, MinnaNoNihongo, Irodori — unless a
    // full-corpus seed requested skipping them (app progress excludes
    // already-studied sets from the import).
    if (options.skipApps !== true) {
        await selectDefaultApps(page);
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

    if (options.stopAtSummary === true) {
        return true;
    }

    // Summary → Import → Scoring
    await page.getByTestId("onboarding-import").click();
    await expect(page.getByTestId("onboarding-import")).toHaveAttribute("data-loading", "true", { timeout: 5000 });
    const scoringTimeout = options.scoringReadyTimeout ?? 120_000;
    await expect(page.getByTestId("onboarding-scoring-step")).toBeVisible({ timeout: scoringTimeout });

    // Wait for scoring step to finish loading cards before returning
    await waitForScoringReady(page, Math.max(scoringTimeout, 30_000));

    return true;
}

/**
 * Selects the default app set (Migii, DuolingoRu, MinnaNoNihongo, Irodori)
 * at the onboarding apps step. Guarded by visibility: an app missing from
 * the step is skipped rather than failing the flow.
 */
async function selectDefaultApps(page: Page): Promise<void> {
    for (const app of ["Migii", "DuolingoRu", "MinnaNoNihongo", "Irodori"]) {
        const checkbox = page.getByTestId(`apps-step-app-${app}-checkbox`);
        if (await checkbox.isVisible().catch(() => false)) {
            await checkbox.click();
        }
    }
}

/**
 * Seeds the full N1 corpus stress user: onboarding at level N1 with NO app
 * progress (app progress would exclude already-studied sets from the
 * import — the stress corpus must be complete), through import and scoring,
 * then marks everything known and finishes onboarding.
 */
export async function completeN1StressSeed(page: Page): Promise<void> {
    await completeOnboardingToScoring(page, {
        level: "N1",
        skipApps: true,
        scoringReadyTimeout: 240_000,
        onboardingUrlTimeout: 90_000,
    });

    const onboarding = new OnboardingPage(page);
    // Mark every remaining card as known (bulk O(1) per card — see
    // scoring_mark_all.rs), then finish onboarding.
    await expect(onboarding.scoringHint).toBeVisible({ timeout: 30_000 });
    await onboarding.clickMarkAllKnown();
    await expect(onboarding.scoringComplete).toBeVisible({ timeout: 240_000 });
    await onboarding.clickFinish();
    // The finish checkpoint save_syncs the full ~11k-card corpus; a debug
    // wasm build grinds the serialization for well over the old 60s.
    await page.waitForURL(/\/home/, { timeout: 240_000 });
}
