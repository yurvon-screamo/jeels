import { expect } from "@playwright/test";
import { Given, When, Then } from "../fixtures";
import { OnboardingPage, PhrasesPage } from "../../pages";
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
    const filledPath = favBtn.locator('svg path[fill="currentColor"]');
    await expect(filledPath).toBeVisible({ timeout: 5_000 });
});

// Generic filter + detail steps

When('выбирает фильтр карточек {string}', async ({ page }, filter: string) => {
    const filterMap: Record<string, string> = {
        "all": "all", "new": "new", "learning": "in-progress",
        "hard": "hard", "learned": "learned",
    };
    const suffix = filterMap[filter] ?? filter;
    const filterBtn = page.getByTestId(new RegExp(`filter-${suffix}`));
    await filterBtn.first().click();
});

When('нажимает кнопку возврата с деталей', async ({ page }) => {
    const backBtn = page.getByTestId(/detail.*back/).or(page.getByTestId("back-btn")).or(page.getByRole("button", { name: /Back|Назад|戻/ }));
    await backBtn.first().click();
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
    const phrasesPage = new PhrasesPage(page);
    await phrasesPage.deleteCancelBtn.click();
});
