import { expect } from "@playwright/test";
import { Given, When, Then } from "../fixtures";
import { GrammarPage } from "../../pages";

Given('у пользователя есть добавленная грамматическая карточка', async ({ page }) => {
    const grammarPage = new GrammarPage(page);
    await grammarPage.goto();
    await expect(grammarPage.grammarPage).toBeVisible({ timeout: 15_000 });
    await grammarPage.openAddModal();
    await grammarPage.selectRule("～ます");
    await grammarPage.addSelectedRules();
    await expect(grammarPage.grammarGrid).toBeVisible({ timeout: 30_000 });
});

When('пользователь открывает страницу грамматики', async ({ page }) => {
    const grammarPage = new GrammarPage(page);
    await grammarPage.goto();
    await expect(grammarPage.grammarPage).toBeVisible({ timeout: 15_000 });
});

When('открывает добавление грамматики', async ({ page }) => {
    const grammarPage = new GrammarPage(page);
    await grammarPage.openAddModal();
});

When('выбирает первый грамматический уровень N5', async ({}) => {
    // N5 is the default level when the drawer opens — no action needed.
});

When('подтверждает добавление грамматики', async ({ page }) => {
    const grammarPage = new GrammarPage(page);
    await grammarPage.selectRule("～ます");
    await grammarPage.addSelectedRules();
});

When('нажимает кнопку выбора всех правил', async ({ page }) => {
    const grammarPage = new GrammarPage(page);
    await grammarPage.selectAllRules();
});

When('выбирает уровни грамматики N5 и N4', async ({ page }) => {
    const grammarPage = new GrammarPage(page);
    await grammarPage.selectLevel("N5");
    await grammarPage.selectAllRules();
    await grammarPage.selectLevel("N4");
    await grammarPage.selectAllRules();
});

Then('грамматическая карточка отображается в сетке', async ({ page }) => {
    const grammarPage = new GrammarPage(page);
    await expect(grammarPage.grammarGrid).toBeVisible({ timeout: 30_000 });
    await expect(grammarPage.emptyState).not.toBeVisible();
});

Then('отображается более одной грамматической карточки', async ({ page }) => {
    const grammarPage = new GrammarPage(page);
    const count = await grammarPage.getCardCount();
    expect(count).toBeGreaterThan(1);
});

Then('на странице грамматики отображается пустое состояние', async ({ page }) => {
    const grammarPage = new GrammarPage(page);
    await expect(grammarPage.emptyState).toBeVisible();
});

When('пользователь удаляет первую грамматическую карточку', async ({ page }) => {
    const grammarPage = new GrammarPage(page);
    await grammarPage.deleteCardByIndex(0);
});

When('пользователь отменяет удаление первой грамматической карточки', async ({ page }) => {
    const grammarPage = new GrammarPage(page);
    await grammarPage.cancelDeleteCardByIndex(0);
});

When('пользователь ищет грамматику {string}', async ({ page }, query: string) => {
    const grammarPage = new GrammarPage(page);
    await grammarPage.searchGrammar(query);
});

Then('грамматическая сетка пуста', async ({ page }) => {
    const grammarPage = new GrammarPage(page);
    await expect(grammarPage.emptyState).toBeVisible({ timeout: 10_000 });
});

When('нажимает кнопку перехода на главную', async ({ page }) => {
    await page.goto("/home");
    await page.waitForURL(/\/home$/, { timeout: 10_000 });
});

When('пользователь отмечает первую грамматику как известную', async ({ page }) => {
    const grammarPage = new GrammarPage(page);
    await grammarPage.markCardAsKnownByIndex(0);
});

When('пользователь открывает детали первой грамматики', async ({ page }) => {
    await page.getByTestId("grammar-card-item").first().click();
    await page.waitForURL(/\/grammar\//, { timeout: 10_000 });
});

Then('отображается страница деталей грамматики', async ({ page }) => {
    await expect(page.getByTestId("grammar-detail-page")).toBeVisible({ timeout: 10_000 });
});

Then('отображается содержимое деталей грамматики', async ({ page }) => {
    await expect(page.getByTestId("grammar-detail-page")).toBeVisible({ timeout: 10_000 });
    await expect(page.getByTestId(/breadcrumb/).first()).toBeVisible();
});

When('нажимает хлебные крошки', async ({ page }) => {
    await page.getByTestId(/breadcrumb/).first().click();
});

Then('отображается кнопа отметки как известное', async ({ page }) => {
    const card = page.getByTestId("grammar-card-item").first();
    await expect(card.getByTestId("grammar-card-item-mark-known-btn")).toBeVisible();
});

Given('у пользователя есть много грамматических карточек', async ({ page }) => {
    const grammarPage = new GrammarPage(page);
    await grammarPage.goto();
    await expect(grammarPage.grammarPage).toBeVisible({ timeout: 15_000 });
    await grammarPage.openAddModal();
    await grammarPage.selectLevel("N5");
    await grammarPage.selectAllRules();
    await grammarPage.selectLevel("N4");
    await grammarPage.selectAllRules();
    await grammarPage.selectLevel("N3");
    await grammarPage.selectAllRules();
    await grammarPage.addSelectedRules();
});

When('нажимает кнопку практики', async ({ page }) => {
    await page.getByTestId("grammar-detail-practice-btn").click();
});

Then('отображается сессия практики с вопросами', async ({ page }) => {
    await expect(page.getByTestId("practice-question")).toBeVisible({ timeout: 15_000 });
});

Then('отображается вопрос практики', async ({ page }) => {
    await expect(page.getByTestId("practice-question")).toBeVisible({ timeout: 15_000 });
});

Then('отображаются варианты ответа практики', async ({ page }) => {
    await expect(page.getByTestId("practice-option").first()).toBeVisible({ timeout: 10_000 });
});

When('отвечает на все вопросы практики', async ({ page }) => {
    for (let i = 0; i < 20; i++) {
        const complete = page.getByTestId("practice-complete");
        if (await complete.isVisible().catch(() => false)) break;
        const option = page.getByTestId("practice-option").first();
        if (await option.isVisible({ timeout: 3_000 }).catch(() => false)) {
            await option.click();
            const nextBtn = page.getByTestId("practice-next-btn");
            if (await nextBtn.isVisible({ timeout: 3_000 }).catch(() => false)) {
                await nextBtn.click();
            }
        } else break;
    }
});

Then('отображается завершение практики', async ({ page }) => {
    await expect(page.getByTestId("practice-complete")).toBeVisible({ timeout: 15_000 });
});
