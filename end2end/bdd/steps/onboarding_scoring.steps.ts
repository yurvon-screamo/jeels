import { expect } from "@playwright/test";
import { Given, When, Then } from "../fixtures";
import { OnboardingPage } from "../../pages";
import { completeOnboardingToScoring } from "../../helpers/onboarding";
import { skipOnboarding } from "../../helpers/navigation";

// Module-scoped snapshot of the question text captured by the "Не знаю" step
// so the "card not shown again" assertion can verify it differs from the
// current question after a reload. Scenario-scoped because playwright-bdd
// runs scenarios sequentially within a file.
let lastDontKnowQuestion: string | null = null;

Given('новый пользователь дошёл до шага оценивания карточек', async ({ page }) => {
    const reached = await completeOnboardingToScoring(page);
    expect(reached).toBeTruthy();
});

Then('отображается вопрос карточки', async ({ page }) => {
    const onboarding = new OnboardingPage(page);
    await expect(onboarding.scoringHint).toBeVisible({ timeout: 10_000 });
    await expect(onboarding.scoringQuestion).toBeVisible({ timeout: 10_000 });
});

Then('отображается вариант ответа', async ({ page }) => {
    const onboarding = new OnboardingPage(page);
    await expect(onboarding.scoringAnswer).toBeVisible();
});

Then('отображается прогресс оценивания', async ({ page }) => {
    const onboarding = new OnboardingPage(page);
    // After a reload the auth-store re-loads all dictionaries on a fresh
    // ProtectedRoute mount; that can take well over the default 5s timeout
    // depending on CDN latency. Give the scoring UI up to 60s to surface.
    await expect(onboarding.scoringProgress).toBeVisible({ timeout: 60_000 });
});

Then('отображаются кнопки "Знаю" и "Не знаю"', async ({ page }) => {
    const onboarding = new OnboardingPage(page);
    await expect(onboarding.scoringKnowBtn).toBeVisible();
    await expect(onboarding.scoringDontKnowBtn).toBeVisible();
});

When('пользователь нажимает "Не знаю"', async ({ page }) => {
    const onboarding = new OnboardingPage(page);
    // Snapshot the question text BEFORE dismissing so the "card not shown
    // again" assertion after a reload can verify the same card does not
    // reappear. We then wait for the question to actually change so the step
    // completes only once the next card (or the completion screen) is in
    // place — without relying on the progress-bar textContent, which is now
    // a stable section-label list rather than a per-click counter.
    lastDontKnowQuestion = await onboarding.scoringQuestion.textContent();
    await onboarding.clickDontKnow();
    for (let i = 0; i < 30; i++) {
        const isComplete = await onboarding.scoringComplete.isVisible().catch(() => false);
        if (isComplete) return;
        const next = await onboarding.scoringQuestion.textContent().catch(() => null);
        if (next !== null && next !== lastDontKnowQuestion) return;
        await page.waitForTimeout(200);
    }
});

When('пользователь перезагружает страницу приложения', async ({ page }) => {
    await page.reload();
});

Then('карточка "Не знаю" не показывается снова', async ({ page }) => {
    // AC #1 verification: skipped-card persistence. After a reload the
    // scoring step must resume from where the user left off, so the card
    // dismissed via "Don't know" (snapshotted in `lastDontKnowQuestion`)
    // must NOT reappear.
    expect(
        lastDontKnowQuestion,
        "snapshot from the previous 'Не знаю' step is required for this assertion",
    ).not.toBeNull();

    const onboarding = new OnboardingPage(page);
    await expect(onboarding.scoringHint).toBeVisible({ timeout: 30_000 });
    await expect(onboarding.scoringQuestion).toBeVisible({ timeout: 30_000 });
    const current = await onboarding.scoringQuestion.textContent();
    expect(current, "scoring question must be loaded after reload").not.toBeNull();
    expect(current, "skipped card must not reappear after reload").not.toBe(
        lastDontKnowQuestion,
    );
});

Then('отображаются метки секций прогресс-бара', async ({ page }) => {
    // Section markers are rendered as separate divs with stable test-ids
    // derived from CardType::sort_order() (Grammar=0, Kanji=1, Vocab=2).
    // The exact set visible depends on which cards the imported sets
    // produced; we assert at least one marker is rendered for the typical
    // onboarding (which always imports Vocabulary cards from jlpt_n5).
    const markers = page.locator('[data-testid^="scoring-progress-marker-"]');
    await expect(markers.first()).toBeVisible({ timeout: 10_000 });
});

When('пользователь нажимает "Знаю"', async ({ page }) => {
    const onboarding = new OnboardingPage(page);
    await onboarding.clickKnow();
    // Wait for either the next card or the complete screen to appear.
    // Using isVisible().catch() polling pattern (not Promise.race+catch)
    // to avoid swallowing errors silently.
    for (let i = 0; i < 30; i++) {
        const isComplete = await onboarding.scoringComplete.isVisible().catch(() => false);
        const hasNext = await onboarding.scoringDontKnowBtn.isVisible().catch(() => false);
        if (isComplete || hasNext) return;
        await page.waitForTimeout(200);
    }
});

When('пользователь отмечает все оставшиеся карточки как известные', async ({ page }) => {
    const onboarding = new OnboardingPage(page);
    await expect(onboarding.scoringHint).toBeVisible({ timeout: 30_000 });
    await onboarding.clickMarkAllKnown();
});

Then('отображается сообщение о завершении оценивания', async ({ page }) => {
    const onboarding = new OnboardingPage(page);
    await expect(onboarding.scoringComplete).toBeVisible({ timeout: 60_000 });
});

Then('отображается кнопка завершения онбординга', async ({ page }) => {
    const onboarding = new OnboardingPage(page);
    await expect(onboarding.finishButton).toBeVisible();
});

When('пользователь нажимает "Пропустить оценивание"', async ({ page }) => {
    const onboarding = new OnboardingPage(page);
    await onboarding.clickSkipScoring();
});

Then('происходит переход на главную страницу', async ({ page }) => {
    await page.waitForURL(/\/home$/, { timeout: 30_000 });
    await expect(page).toHaveURL(/\/home$/);
});

When('нажимает кнопку завершения онбординга', async ({ page }) => {
    const onboarding = new OnboardingPage(page);
    await onboarding.clickFinish();
});

When('пользователь пропускает онбординг', async ({ page }) => {
    await skipOnboarding(page);
});

When('пользователь доходит до шага приложений онбординга', async ({ page }) => {
    const onboarding = new OnboardingPage(page);
    await onboarding.goToNextStep();
    await onboarding.goToNextStep();
    await page.getByTestId("jlpt-option-n4").click();
    await onboarding.goToNextStep();
    await expect(onboarding.appsStep).toBeVisible({ timeout: 10_000 });
});

When('нажимает кнопку "Назад" в онбординге', async ({ page }) => {
    const onboarding = new OnboardingPage(page);
    await onboarding.goToPrevStep();
});

Then('отображается шаг выбора уровня JLPT', async ({ page }) => {
    await expect(page.getByTestId("onboarding-jlpt-step")).toBeVisible({ timeout: 10_000 });
});

When('пользователь нажимает "Знаю все"', async ({ page }) => {
    const onboarding = new OnboardingPage(page);
    await expect(onboarding.scoringHint).toBeVisible({ timeout: 30_000 });
    await page.getByTestId("onboarding-mark-all-known").click();
});

Then('отображается модальное окно подтверждения', async ({ page }) => {
    await expect(page.getByTestId("onboarding-confirm")).toBeVisible({ timeout: 10_000 });
});

Then('отображается кнопка подтверждения', async ({ page }) => {
    await expect(page.getByTestId("onboarding-confirm-ok")).toBeVisible();
});

Then('отображается кнопка отмены', async ({ page }) => {
    await expect(page.getByTestId("onboarding-confirm-cancel")).toBeVisible();
});

When('нажимает кнопку отмены в модальном окне', async ({ page }) => {
    await page.getByTestId("onboarding-confirm-cancel").click();
});

Then('модальное окно подтверждения не отображается', async ({ page }) => {
    await expect(page.getByTestId("onboarding-confirm")).not.toBeVisible();
});

When('пользователь подтверждает действие в модальном окне', async ({ page }) => {
    await page.getByTestId("onboarding-confirm-ok").click();
});
