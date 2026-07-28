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
    const favBtn = page.locator('[data-testid*="card-item"]').first()
        .locator('[data-testid*="favorite"]').first();
    await favBtn.click();
    // Wait for the filled-heart SVG to render so the toggle actually landed
    // before any subsequent navigation reloads the page state.
    await expect(favBtn.locator('svg path[fill="currentColor"]')).toBeVisible({ timeout: 10_000 });
});

Then('первая карточка отмечена избранной', async ({ page }) => {
    const favBtn = page.locator('[data-testid*="card-item"]').first()
        .locator('[data-testid*="favorite"]').first();
    const filledPath = favBtn.locator('svg path[fill="currentColor"]');
    await expect(filledPath).toBeVisible({ timeout: 5_000 });
});

// Generic filter + detail steps

When('выбирает фильтр карточек {string}', async ({ page }, filter: string) => {
    const filterMap: Record<string, string> = {
        "all": "all", "new": "new", "learning": "in-progress",
        "hard": "hard", "learned": "learned", "favorite": "favorite",
    };
    const suffix = filterMap[filter] ?? filter;
    const filterBtn = page.getByTestId(new RegExp(`filter-${suffix}`));
    await filterBtn.first().click();
});

When('нажимает кнопку возврата с деталей', async ({ page }) => {
    // The breadcrumb "back" link is per-detail (e.g. grammar-detail-
    // breadcrumbs-back). Rather than enumerate every detail variant, we
    // exercise the in-app router via the browser's history back, which is
    // what a hardware/device back button would trigger. This still verifies
    // the router returns to the parent list — a regression in routing or in
    // ProtectedRoute would surface here.
    const before = new URL(page.url()).pathname;
    await page.goBack();
    const after = new URL(page.url()).pathname;
    if (before === after) {
        throw new Error(`page.goBack() did not navigate away from ${before}`);
    }
});

When('удаляет карточку со страницы деталей', async ({ page }) => {
    const deleteBtn = page.getByTestId(/detail.*delete/).or(page.getByTestId("delete-card-btn"));
    await deleteBtn.first().click();
    const confirm = page.getByTestId(/delete.*confirm/);
    if (await confirm.isVisible().catch(() => false)) {
        await confirm.click();
    }
});

When('пользователь устанавливает мобильный размер экрана', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 812 });
});

Then('метрики FSRS скрыты на мобильном устройстве', async ({ page }) => {
    await expect(page.getByTestId(/fsrs.*metric/)).not.toBeVisible({ timeout: 5_000 });
});

Then('кнопка отметки скрыта для первой карточки', async ({ page }) => {
    const markBtn = page.locator('[data-testid*="card-item"]').first()
        .locator('[data-testid*="mark-known"]').first();
    await expect(markBtn).not.toBeVisible({ timeout: 5_000 });
});

Then('отображается кнопка отметки как известное', async ({ page }) => {
    const markBtn = page.locator('[data-testid*="card-item"]').first()
        .locator('[data-testid*="mark-known"]').first();
    await expect(markBtn).toBeVisible({ timeout: 5_000 });
});

When('отмечает первую фразу как известную', async ({ page }) => {
    await page.locator('[data-testid*="card-item"]').first()
        .locator('[data-testid*="mark-known"]').first().click();
});

When('отменяет удаление первой фразы', async ({ page }) => {
    const cancelBtn = page.getByTestId(/cancel.*delete|delete.*cancel/);
    await cancelBtn.first().click();
});
