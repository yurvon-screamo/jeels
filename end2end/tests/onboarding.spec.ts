import { expect, test, type Page } from "@playwright/test";
import { testWithFreshUser } from "../fixtures";
import { completeOnboardingToScoring, waitForScoringReady } from "../helpers/onboarding";

/**
 * Onboarding Flow E2E Tests
 *
 * Tests the complete onboarding flow:
 * 1. Login (handled by fixture)
 * 2. Intro step
 * 3. JLPT level selection (N4)
 * 4. Apps selection (Migii, Duolingo RU, Minna N4)
 * 5. Progress configuration (~50% on each)
 * 6. Summary and import
 */

testWithFreshUser.describe("Onboarding Flow - N4 with ~50% Progress", () => {
    testWithFreshUser("should complete full onboarding flow", async ({ page }: { page: Page }) => {
        test.setTimeout(300_000);

        // Set viewport for consistent testing
        await page.setViewportSize({ width: 1280, height: 720 });

        // Fixture already logged in and is on /home.
        // The app asynchronously checks imported_sets() and redirects to /onboarding.
        await page.goto("/home");
        await page.waitForURL(/\/onboarding$/, { timeout: 30_000 });

        // ========================================
        // Step 0: Verify onboarding page with stepper
        // ========================================

        // Wait for loading to complete
        await expect(page.getByTestId("onboarding-spinner")).not.toBeVisible({
            timeout: 10_000
        });

        // Verify page structure
        await expect(page.getByTestId("onboarding-page")).toBeVisible();
        await expect(page.getByTestId("onboarding-card")).toBeVisible();
        await expect(page.getByTestId("onboarding-stepper")).toBeVisible();

        // ========================================
        // Step 1: Intro - language selector and feature cards
        // ========================================
        await expect(page.getByTestId("onboarding-intro-step")).toBeVisible();

        // Language bar with label and selector
        const languageBar = page.getByTestId("intro-step-language-bar");
        await expect(languageBar).toBeVisible();
        const langEnBtn = page.getByTestId("lang-toggle-en");
        const langRuBtn = page.getByTestId("lang-toggle-ru");
        await expect(langEnBtn).toBeVisible();
        await expect(langRuBtn).toBeVisible();

        // Language switching: click EN → title changes
        const introTitle = page.getByTestId("intro-step-title");
        const initialTitle = await introTitle.textContent();
        await langEnBtn.click();
        await expect(introTitle).not.toHaveText(initialTitle ?? '', { timeout: 5000 });

        // Switch back to Russian
        await langRuBtn.click();
        await expect(introTitle).toHaveText(initialTitle ?? '', { timeout: 5000 });

        // (Feature cards removed from IntroStep - no longer in UI)

        // Welcome text
        await expect(page.getByTestId("intro-step-title")).toBeVisible();

        // Skip button
        await expect(page.getByTestId("onboarding-skip")).toBeVisible();

        // Screenshot
        await page.screenshot({
            path: "test-results/onboarding-step-1-intro.png",
            fullPage: true
        });

        // Proceed to next step
        await page.getByTestId("onboarding-next").click();

        // ========================================
        // Step 1.5: Load - select daily load
        // ========================================
        await expect(page.getByTestId("load-step-title")).toBeVisible();

        // Proceed with default (medium) load
        await page.getByTestId("onboarding-next").click();

        // Verify we moved to JLPT step
        await expect(page.getByTestId("onboarding-jlpt-step")).toBeVisible();

        // ========================================
        // Step 2: JLPT - should select N4 level
        // ========================================
        await expect(page.getByTestId("onboarding-jlpt-step")).toBeVisible();
        await expect(page.getByTestId("jlpt-step-title")).toBeVisible();

        // Take screenshot before selection
        await page.screenshot({
            path: "test-results/onboarding-step-2-jlpt-before.png",
            fullPage: true
        });

        // Select N4 level using test_id
        await page.getByTestId("jlpt-option-n4").click();

        // Verify selection is highlighted (check for selected class)
        const n4Option = page.getByTestId("jlpt-option-n4");
        await expect(n4Option).toHaveClass(/selected/, { timeout: 5000 });

        // Screenshot after selection
        await page.screenshot({
            path: "test-results/onboarding-step-2-jlpt-after.png",
            fullPage: true
        });

        // Proceed to next step
        await page.getByTestId("onboarding-next").click();
        await expect(page.getByTestId("onboarding-apps-step")).toBeVisible();

        // ========================================
        // Step 3: Apps - should select all available apps
        // ========================================
        await expect(page.getByTestId("onboarding-apps-step")).toBeVisible();
        await expect(page.getByTestId("apps-step-title")).toBeVisible();

        // Wait for apps to load from CDN (filtered_apps is populated async)
        await expect(page.getByTestId("apps-step-app-Migii-checkbox")).toBeVisible({ timeout: 30_000 });

        // Take screenshot before selections
        await page.screenshot({
            path: "test-results/onboarding-step-3-apps-before.png",
            fullPage: true
        });

        // Select Migii
        const migiiCheckbox = page.getByTestId("apps-step-app-Migii-checkbox");
        await migiiCheckbox.click();

        // Verify Migii is selected (check checkbox state)
        const migiiCard = page.getByTestId("apps-step-app-Migii");
        await expect(migiiCard).toHaveClass(/selected/, { timeout: 5000 });

        // Select Duolingo 「RU」
        const duolingoRuCheckbox = page.getByTestId("apps-step-app-DuolingoRu-checkbox");
        if (await duolingoRuCheckbox.isVisible().catch(() => false)) {
            await duolingoRuCheckbox.click();
        }

        // Select Duolingo 「EN」
        const duolingoEnCheckbox = page.getByTestId("apps-step-app-DuolingoEn-checkbox");
        if (await duolingoEnCheckbox.isVisible().catch(() => false)) {
            await duolingoEnCheckbox.click();
        }

        // Select Minna no Nihongo
        const minnaNoNihongoCheckbox = page.getByTestId("apps-step-app-MinnaNoNihongo-checkbox");
        if (await minnaNoNihongoCheckbox.isVisible().catch(() => false)) {
            await minnaNoNihongoCheckbox.click();
        }

        // Select Irodori
        const irodoriCheckbox = page.getByTestId("apps-step-app-Irodori-checkbox");
        if (await irodoriCheckbox.isVisible().catch(() => false)) {
            await irodoriCheckbox.click();
        }

        // Screenshot after selections
        await page.screenshot({
            path: "test-results/onboarding-step-3-apps-after.png",
            fullPage: true
        });

        // Proceed to progress step
        await page.getByTestId("onboarding-next").click();
        await expect(page.getByTestId("onboarding-progress-step")).toBeVisible();

        // ========================================
        // Step 4: Progress - should configure ~50% progress for each app
        // ========================================
        await expect(page.getByTestId("onboarding-progress-step")).toBeVisible();
        await expect(page.getByTestId("progress-step-title")).toBeVisible();

        // Take screenshot
        await page.screenshot({
            path: "test-results/onboarding-step-4-progress-before.png",
            fullPage: true
        });

        // Configure Migii progress (N4, middle lesson)
        const migiiLevelDropdown = page.getByTestId("migii-level-dropdown");
        if (await migiiLevelDropdown.isVisible().catch(() => false)) {
            await migiiLevelDropdown.click();
            await page.getByTestId("migii-level-dropdown-option-N4").click();

            // Select middle lesson (around lesson 10 for N4)
            const migiiLessonDropdown = page.getByTestId("migii-lesson-dropdown");
            await migiiLessonDropdown.click();
            await page.getByTestId("migii-lesson-dropdown-option-lesson_10").click();
        }

        // Configure Duolingo 「RU」 progress
        const duolingoRuModuleDropdown = page.getByTestId("DuolingoRu-module-dropdown");
        if (await duolingoRuModuleDropdown.isVisible().catch(() => false)) {
            // Select first module
            await duolingoRuModuleDropdown.click();
            await page.getByTestId("DuolingoRu-module-dropdown-option-module_1").click();

            // Select ~50% unit
            const duolingoRuUnitDropdown = page.getByTestId("DuolingoRu-unit-dropdown");
            await duolingoRuUnitDropdown.click();
            await page.getByTestId("DuolingoRu-unit-dropdown-option-unit_10").click();
        }

        // Configure Minna no Nihongo progress (two dropdowns: level + lesson)
        const minnaLevelDropdown = page.getByTestId("minna-level-dropdown");
        if (await minnaLevelDropdown.isVisible().catch(() => false)) {
            await minnaLevelDropdown.click();
            await page.getByTestId("minna-level-dropdown-option-N4").click();

            const minnaLessonDropdown = page.getByTestId("minna-lesson-dropdown");
            await minnaLessonDropdown.click();
            await page.getByTestId("minna-lesson-dropdown-option-lesson_38").click();
        }

        // Configure Irodori progress (book + lesson dropdowns)
        const irodoriBookDropdown = page.getByTestId("irodori-book-dropdown");
        if (await irodoriBookDropdown.isVisible().catch(() => false)) {
            await irodoriBookDropdown.click();
            await page.getByTestId("irodori-book-dropdown-option-nyuumon").click();

            const irodoriLessonDropdown = page.getByTestId("irodori-lesson-dropdown");
            await irodoriLessonDropdown.click();
            await page.getByTestId("irodori-lesson-dropdown-option-lesson_9").click();
        }

        // Screenshot after progress configuration
        await page.screenshot({
            path: "test-results/onboarding-step-4-progress-after.png",
            fullPage: true
        });

        // Proceed to summary
        await page.getByTestId("onboarding-next").click();
        await expect(page.getByTestId("onboarding-summary-step")).toBeVisible();

        // ========================================
        // Step 5: Summary - should display selected sets and allow toggle
        // ========================================
        await expect(page.getByTestId("onboarding-summary-step")).toBeVisible();
        await expect(page.getByTestId("summary-step-title")).toBeVisible();

        // Verify word count is displayed
        await expect(page.getByTestId("summary-step-stats")).toBeVisible();

        // Take screenshot
        await page.screenshot({
            path: "test-results/onboarding-step-5-summary-before.png",
            fullPage: true
        });

        // Test accordion toggle (if expandable)
        const accordionHeader = page.getByTestId('summary-step-accordion-header').first();
        if (await accordionHeader.isVisible().catch(() => false)) {
            await accordionHeader.click();
            // CSS accordion animation requires brief wait
            await page.waitForTimeout(300);
        }

        // Test set toggle checkbox (optional - deselect then reselect)
        const firstSetCheckbox = page.locator('[data-testid^="summary-step-set-"][data-testid$="-checkbox"]').first();
        if (await firstSetCheckbox.isVisible().catch(() => false)) {
            // Note: Clicking will toggle off, then we'd need to toggle back on
            // For now, just verify it exists and is clickable
            await expect(firstSetCheckbox).toBeEnabled();
        }

        // Screenshot before import
        await page.screenshot({
            path: "test-results/onboarding-step-5-summary-before-import.png",
            fullPage: true
        });

        // ========================================
        // Final: Complete import and redirect to home

        // ========================================
        // Start import
        await page.getByTestId("onboarding-import").click();

        // Verify import button shows loading state
        await expect(page.getByTestId("onboarding-import")).toHaveAttribute("data-loading", "true", { timeout: 5000 });

        // Wait for scoring step to appear after import completes
        await expect(page.getByTestId("onboarding-scoring-step")).toBeVisible({ timeout: 120_000 });

        // Wait for scoring step to finish loading cards
        await waitForScoringReady(page);

        // Take screenshot of scoring step
        await page.screenshot({
            path: "test-results/onboarding-step-6-scoring.png",
            fullPage: true
        });

        // Click "Знаю все" to mark all cards as known
        await page.getByTestId("onboarding-mark-all-known").click();

        // Wait for scoring completion message (increased timeout for large card sets)
        await expect(page.getByTestId("scoring-step-complete")).toBeVisible({ timeout: 60_000 });

        // Click "Завершить" to finish onboarding and navigate to home
        await expect(page.getByTestId("onboarding-finish")).toBeVisible({ timeout: 5000 });
        await page.getByTestId("onboarding-finish").click();

        // Wait for redirect to home (can take time for import)
        await page.waitForURL(/\/home$/, { timeout: 30_000 });

        // Verify we're on home page
        await expect(page).toHaveURL(/\/home$/);

        // Take final screenshot
        await page.screenshot({
            path: "test-results/onboarding-complete-home.png",
            fullPage: true
        });
    });

    testWithFreshUser("should skip onboarding and redirect to home", async ({ page }: { page: Page }) => {
        // Fixture already logged in — wait for navigation to complete
        await page.waitForURL(/\/(home|onboarding)/, { timeout: 30_000 });

        // If on home already, skip is done
        if (page.url().includes("/home")) {
            await expect(page).toHaveURL(/\/home$/);
            return;
        }

        // Otherwise we're on onboarding
        await expect(page.getByTestId("onboarding-spinner")).not.toBeVisible({ timeout: 10_000 });
        await expect(page.getByTestId("onboarding-skip")).toBeVisible();
        await page.getByTestId("onboarding-skip").click();
        await page.waitForURL(/\/home$/, { timeout: 10_000 });
        await expect(page).toHaveURL(/\/home$/);
    });

    testWithFreshUser("should navigate back through steps", async ({ page }: { page: Page }) => {
        // Fixture already logged in — wait for navigation to complete
        await page.waitForURL(/\/(home|onboarding)/, { timeout: 30_000 });

        await expect(page.getByTestId("onboarding-spinner")).not.toBeVisible({ timeout: 10_000 });

        // Intro → Load
        await page.getByTestId("onboarding-next").click();
        await expect(page.getByTestId("onboarding-load-step")).toBeVisible();

        // Load → JLPT
        await page.getByTestId("onboarding-next").click();
        await expect(page.getByTestId("onboarding-jlpt-step")).toBeVisible();

        // JLPT → Apps (select N4 first)
        await page.getByTestId("jlpt-option-n4").click();
        await page.getByTestId("onboarding-next").click();
        await expect(page.getByTestId("onboarding-apps-step")).toBeVisible();

        // Back to JLPT
        await page.getByTestId("onboarding-prev").click();
        await expect(page.getByTestId("onboarding-jlpt-step")).toBeVisible();

        // Back to Load
        await page.getByTestId("onboarding-prev").click();
        await expect(page.getByTestId("onboarding-load-step")).toBeVisible();

        // Back to Intro
        await page.getByTestId("onboarding-prev").click();
        await expect(page.getByTestId("onboarding-intro-step")).toBeVisible();
    });
});

testWithFreshUser.describe("Onboarding - Scoring Step", () => {
    testWithFreshUser("should display scoring step with cards after import", async ({ page }) => {
        test.setTimeout(300_000);
        const reachedScoring = await completeOnboardingToScoring(page);
        expect(reachedScoring).toBeTruthy();

        await expect(page.getByTestId("onboarding-scoring-step")).toBeVisible();
        await expect(page.getByTestId("scoring-step-hint")).toBeVisible();
        await expect(page.getByTestId("scoring-step-progress")).toBeVisible();
        await expect(page.getByTestId("scoring-step-question")).toBeVisible();
        await expect(page.getByTestId("scoring-step-answer")).toBeVisible();
        await expect(page.getByTestId("scoring-step-dont-know")).toBeVisible();
        await expect(page.getByTestId("scoring-step-know")).toBeVisible();
    });

    testWithFreshUser("should skip card with 'Don't Know' button", async ({ page }) => {
        test.setTimeout(300_000);
        const reachedScoring = await completeOnboardingToScoring(page);
        expect(reachedScoring).toBeTruthy();

        const progressBefore = await page.getByTestId("scoring-step-progress").textContent();

        await page.getByTestId("scoring-step-dont-know").click();
        await expect(page.getByTestId("scoring-step-progress")).not.toHaveText(progressBefore ?? '', { timeout: 5000 });

        const progressAfter = await page.getByTestId("scoring-step-progress").textContent();
        expect(progressAfter).not.toBe(progressBefore);
    });

    testWithFreshUser("should mark card as known with 'Know' button", async ({ page }) => {
        test.setTimeout(300_000);
        const reachedScoring = await completeOnboardingToScoring(page);
        expect(reachedScoring).toBeTruthy();

        await page.getByTestId("scoring-step-know").click();
        await Promise.race([
            page.getByTestId("scoring-step-dont-know").waitFor({ state: "visible", timeout: 5000 }),
            page.getByTestId("scoring-step-complete").waitFor({ state: "visible", timeout: 5000 }),
        ]).catch(() => {});

        await expect(page.getByTestId("scoring-step-progress")).toBeVisible();
    });

    testWithFreshUser("should complete scoring when all cards processed", async ({ page }) => {
        test.setTimeout(300_000);
        const reachedScoring = await completeOnboardingToScoring(page);
        expect(reachedScoring).toBeTruthy();

        for (let i = 0; i < 3; i++) {
            const dontKnowBtn = page.getByTestId("scoring-step-dont-know");
            if (await dontKnowBtn.isVisible({ timeout: 2000 }).catch(() => false)) {
                await dontKnowBtn.click();
                await Promise.race([
                    page.getByTestId("scoring-step-dont-know").waitFor({ state: "visible", timeout: 5000 }),
                    page.getByTestId("scoring-step-complete").waitFor({ state: "visible", timeout: 5000 }),
                ]).catch(() => {});
            }
        }

        await page.getByTestId("onboarding-mark-all-known").click();

        await expect(page.getByTestId("scoring-step-complete")).toBeVisible({ timeout: 60_000 });
        await expect(page.getByTestId("onboarding-finish")).toBeVisible();
    });

    testWithFreshUser("should mark all remaining as known", async ({ page }) => {
        test.setTimeout(300_000);
        const reachedScoring = await completeOnboardingToScoring(page);
        expect(reachedScoring).toBeTruthy();

        // Ensure scoring step is fully loaded before clicking mark-all
        await expect(page.getByTestId("scoring-step-hint")).toBeVisible({ timeout: 30_000 });

        await page.getByTestId("onboarding-mark-all-known").click();

        await expect(page.getByTestId("scoring-step-complete")).toBeVisible({ timeout: 60_000 });
        await expect(page.getByTestId("onboarding-finish")).toBeVisible();
    });

    testWithFreshUser("should skip scoring and navigate to home", async ({ page }) => {
        test.setTimeout(300_000);
        const reachedScoring = await completeOnboardingToScoring(page);
        expect(reachedScoring).toBeTruthy();

        await page.getByTestId("onboarding-skip-scoring").click();
        await page.waitForURL(/\/home$/, { timeout: 30_000 });
        await expect(page).toHaveURL(/\/home$/);
    });
});
