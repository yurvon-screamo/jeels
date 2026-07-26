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

When('нажимает кнопку выбора всех правил', async ({ page }) => {
    const grammarPage = new GrammarPage(page);
    await grammarPage.drawerSelectAllBtn.click();
});

When('нажимает хлебные крошки', async ({ page }) => {
    await page.getByTestId(/breadcrumb/).first().click();
});

Then('отображается кнопка отметки как известное', async ({ page }) => {
    const markBtn = page.locator('[data-testid*="card-item"]').first()
        .locator('[data-testid*="mark-as-known"]').first();
    await expect(markBtn).toBeVisible();
});

Given('у пользователя есть много грамматических карточек', async ({ page }) => {
    const grammarPage = new GrammarPage(page);
    await grammarPage.goto();
    await expect(grammarPage.grammarPage).toBeVisible({ timeout: 15_000 });
    await grammarPage.addBtn.click();
    await expect(grammarPage.drawer).toBeVisible({ timeout: 10_000 });
    await grammarPage.drawerAddBtn.click();
    await expect(grammarPage.drawer).not.toBeVisible({ timeout: 60_000 });
});

When('нажимает кнопку практики', async ({ page }) => {
    await page.getByTestId(/practice.*btn|btn.*practice/).first().click();
});

Then('отображается сессия практики с вопросами', async ({ page }) => {
    await expect(page.getByTestId(/practice.*question|practice.*session/).first()).toBeVisible({ timeout: 15_000 });
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

When('пользователь отмечает первую грамматику как известную', async ({ page }) => {
    await page.getByTestId("grammar-card-item").first().locator('[data-testid*="mark-as-known"]').click();
});

When('пользователь открывает детали первой грамматики', async ({ page }) => {
    await page.getByTestId("grammar-card-item").first().click();
    await page.waitForURL(/\/grammar\//, { timeout: 10_000 });
});

Then('отображается страница деталей грамматики', async ({ page }) => {
    await expect(page.getByTestId("grammar-detail-page")).toBeVisible({ timeout: 10_000 });
});
