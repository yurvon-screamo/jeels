import { expect } from "@playwright/test";
import { Given, When, Then } from "../fixtures";
import { OnboardingPage } from "../../pages";
import { completeOnboardingToScoring } from "../../helpers/onboarding";
import { skipOnboarding } from "../../helpers/navigation";

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
    await expect(onboarding.scoringProgress).toBeVisible();
});

Then('отображаются кнопки "Знаю" и "Не знаю"', async ({ page }) => {
    const onboarding = new OnboardingPage(page);
    await expect(onboarding.scoringKnowBtn).toBeVisible();
    await expect(onboarding.scoringDontKnowBtn).toBeVisible();
});

When('пользователь нажимает "Не знаю"', async ({ page }) => {
    const onboarding = new OnboardingPage(page);
    const progressBefore = await onboarding.getScoringProgress();
    await onboarding.clickDontKnow();
    await expect(onboarding.scoringProgress).not.toHaveText(progressBefore, { timeout: 5_000 });
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
