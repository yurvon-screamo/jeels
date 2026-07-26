import { expect } from "@playwright/test";
import { Given, When, Then } from "../fixtures";
import { GrammarPage } from "../../pages";

Given('у пользователя есть добавленная грамматическая карточка', async ({ page }) => {
    const grammarPage = new GrammarPage(page);
    await grammarPage.goto();
    await expect(grammarPage.grammarPage).toBeVisible({ timeout: 15_000 });
    await grammarPage.addBtn.click();
    await expect(grammarPage.drawer).toBeVisible({ timeout: 10_000 });
    await grammarPage.drawerAddBtn.click();
    await expect(grammarPage.grammarGrid).toBeVisible({ timeout: 30_000 });
});

When('пользователь открывает страницу грамматики', async ({ page }) => {
    const grammarPage = new GrammarPage(page);
    await grammarPage.goto();
    await expect(grammarPage.grammarPage).toBeVisible({ timeout: 15_000 });
});

When('открывает добавление грамматики', async ({ page }) => {
    const grammarPage = new GrammarPage(page);
    await grammarPage.addBtn.click();
    await expect(grammarPage.drawer).toBeVisible({ timeout: 10_000 });
});

When('выбирает первый грамматический уровень N5', async ({ page }) => {
    const n5Option = page.getByTestId("drawer-level-N5").first();
    if (await n5Option.isVisible().catch(() => false)) {
        await n5Option.click();
    }
});

When('подтверждает добавление грамматики', async ({ page }) => {
    const grammarPage = new GrammarPage(page);
    await grammarPage.drawerAddBtn.click();
    await expect(grammarPage.drawer).not.toBeVisible({ timeout: 30_000 });
});

Then('грамматическая карточка отображается в сетке', async ({ page }) => {
    const grammarPage = new GrammarPage(page);
    await expect(grammarPage.grammarGrid).toBeVisible({ timeout: 30_000 });
    await expect(grammarPage.emptyState).not.toBeVisible();
});

Then('на странице грамматики отображается пустое состояние', async ({ page }) => {
    const grammarPage = new GrammarPage(page);
    await expect(grammarPage.emptyState).toBeVisible();
});

When('пользователь удаляет первую грамматическую карточку', async ({ page }) => {
    const grammarPage = new GrammarPage(page);
    await page.getByTestId("grammar-card-item").first().locator('[data-testid*="delete"]').first().click();
    await expect(grammarPage.deleteModal).toBeVisible();
    await grammarPage.deleteConfirmBtn.click();
    await expect(grammarPage.deleteModal).not.toBeVisible({ timeout: 10_000 });
});

When('пользователь отменяет удаление первой грамматической карточки', async ({ page }) => {
    const grammarPage = new GrammarPage(page);
    await page.getByTestId("grammar-card-item").first().locator('[data-testid*="delete"]').first().click();
    await expect(grammarPage.deleteModal).toBeVisible();
    await grammarPage.deleteCancelBtn.click();
    await expect(grammarPage.deleteModal).not.toBeVisible();
});

When('пользователь ищет грамматику {string}', async ({ page }, query: string) => {
    const grammarPage = new GrammarPage(page);
    await grammarPage.searchInput.fill(query);
});

Then('грамматическая сетка пуста', async ({ page }) => {
    const grammarPage = new GrammarPage(page);
    await expect(grammarPage.grammarGrid).not.toBeVisible({ timeout: 10_000 });
});

When('нажимает кнопку перехода на главную', async ({ page }) => {
    const grammarPage = new GrammarPage(page);
    await grammarPage.backBtn.click();
});
