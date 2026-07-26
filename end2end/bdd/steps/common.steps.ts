import { expect } from "@playwright/test";
import { Given, Then } from "../fixtures";
import { OnboardingPage } from "../../pages";
import { skipOnboarding } from "../../helpers/navigation";

Given('новый пользователь', async ({ page }) => {
    // The BDD fixture handles user creation + UI login.
    // Wait for the app to settle on either /home or /onboarding.
    await page.waitForURL(/\/(home|onboarding)/, { timeout: 30_000 });
});

Given('пользователь пропустил онбординг', async ({ page }) => {
    await skipOnboarding(page);
});

Then('отображается страница онбординга', async ({ page }) => {
    const onboardingPage = new OnboardingPage(page);
    await expect(onboardingPage.onboardingSpinner).not.toBeVisible({
        timeout: 10_000,
    });
    await onboardingPage.expectOnboardingVisible();
});
