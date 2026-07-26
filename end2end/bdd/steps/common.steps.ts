import { expect } from "@playwright/test";
import { Given, When, Then } from "../fixtures";
import { OnboardingPage } from "../../pages";
import { skipOnboarding } from "../../helpers/navigation";

Given('новый пользователь', async ({ page }) => {
    await page.waitForURL(/\/(home|onboarding)/, { timeout: 30_000 });
});

Given('пользователь пропустил онбординг', async ({ page }) => {
    await skipOnboarding(page);
});

Then('отображается страница онбординга', async ({ page }) => {
    const onboardingPage = new OnboardingPage(page);
    await expect(onboardingPage.onboardingSpinner).not.toBeVisible({ timeout: 10_000 });
    await onboardingPage.expectOnboardingVisible();
});

// Generic CRUD steps — work across any card-list page

Then('кнопка загрузки ещё не отображается', async ({ page }) => {
    await expect(page.getByTestId(/load-more/)).not.toBeVisible({ timeout: 5_000 });
});

Then('кнопка загрузки ещё отображается', async ({ page }) => {
    await expect(page.getByTestId(/load-more/)).toBeVisible({ timeout: 10_000 });
});

When('нажимает кнопку загрузки ещё', async ({ page }) => {
    await page.getByTestId(/load-more/).click();
});

When('переключает избранное первой карточки', async ({ page }) => {
    await page.locator('[data-testid*="card-item"]').first()
        .locator('[data-testid*="favorite"]').first().click();
});

Then('первая карточка отмечена избранной', async ({ page }) => {
    const favBtn = page.locator('[data-testid*="card-item"]').first()
        .locator('[data-testid*="favorite"]').first();
    await expect(favBtn).toHaveClass(/active|selected|favorited/, { timeout: 5_000 });
});
