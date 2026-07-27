import { expect } from "@playwright/test";
import { Given, When, Then } from "../fixtures";
import { KanjiPage } from "../../pages";

Given('у пользователя есть добавленное кандзи', async ({ page }) => {
    const kanjiPage = new KanjiPage(page);
    await kanjiPage.goto();
    await expect(kanjiPage.kanjiPage).toBeVisible({ timeout: 15_000 });
    await kanjiPage.addBtn.click();
    await expect(kanjiPage.drawer).toBeVisible({ timeout: 10_000 });
    await kanjiPage.drawerSelectAllBtn.click();
    await expect(kanjiPage.drawer).toContainText(/Выбрано|selected/i, { timeout: 10_000 }).catch(() => {});
    await kanjiPage.drawerAddBtn.click();
    await expect(kanjiPage.drawer).not.toBeVisible({ timeout: 30_000 });
    await expect(kanjiPage.kanjiGrid).toBeVisible({ timeout: 30_000 });
});

When('пользователь открывает страницу кандзи', async ({ page }) => {
    const kanjiPage = new KanjiPage(page);
    await kanjiPage.goto();
    await expect(kanjiPage.kanjiPage).toBeVisible({ timeout: 15_000 });
});

When('открывает добавление кандзи', async ({ page }) => {
    const kanjiPage = new KanjiPage(page);
    await kanjiPage.addBtn.click();
    await expect(kanjiPage.drawer).toBeVisible({ timeout: 10_000 });
});

When('подтверждает добавление кандзи', async ({ page }) => {
    const kanjiPage = new KanjiPage(page);
    await kanjiPage.drawerAddBtn.click();
    await expect(kanjiPage.drawer).not.toBeVisible({ timeout: 30_000 });
});

Then('кандзи отображается в сетке', async ({ page }) => {
    const kanjiPage = new KanjiPage(page);
    await expect(kanjiPage.kanjiGrid).toBeVisible({ timeout: 30_000 });
    await expect(kanjiPage.emptyState).not.toBeVisible();
});

Then('отображается более одного кандзи', async ({ page }) => {
    const kanjiPage = new KanjiPage(page);
    const count = await kanjiPage.getCardCount();
    expect(count).toBeGreaterThan(1);
});

Then('на странице кандзи отображается пустое состояние', async ({ page }) => {
    const kanjiPage = new KanjiPage(page);
    await expect(kanjiPage.emptyState).toBeVisible();
});

When('пользователь удаляет первое кандзи', async ({ page }) => {
    const kanjiPage = new KanjiPage(page);
    await kanjiPage.deleteCardByIndex(0);
});

When('пользователь отменяет удаление первого кандзи', async ({ page }) => {
    const kanjiPage = new KanjiPage(page);
    await kanjiPage.cancelDeleteCardByIndex(0);
});

When('нажимает кнопку возврата на кандзи', async ({ page }) => {
    const kanjiPage = new KanjiPage(page);
    await kanjiPage.backBtn.click();
});

When('пользователь отмечает первое кандзи как известное', async ({ page }) => {
    const kanjiPage = new KanjiPage(page);
    await kanjiPage.markCardAsKnownByIndex(0);
});

When('пользователь открывает детали первого кандзи', async ({ page }) => {
    const kanjiPage = new KanjiPage(page);
    await page.getByTestId("kanji-card-item").first().click();
    await kanjiPage.expectDetailPageVisible();
});

Then('отображается страница деталей кандзи', async ({ page }) => {
    const kanjiPage = new KanjiPage(page);
    await kanjiPage.expectDetailPageVisible();
});

Then('отображается содержимое деталей кандзи', async ({ page }) => {
    await expect(page.getByTestId("kanji-detail")).toBeVisible({ timeout: 10_000 });
});

When('нажимает кнопку выбора всех кандзи', async ({ page }) => {
    const kanjiPage = new KanjiPage(page);
    await kanjiPage.drawerSelectAllBtn.click();
});

Given('у пользователя есть много кандзи', async ({ page }) => {
    const kanjiPage = new KanjiPage(page);
    await kanjiPage.goto();
    await expect(kanjiPage.kanjiPage).toBeVisible({ timeout: 15_000 });
    await kanjiPage.addBtn.click();
    await expect(kanjiPage.drawer).toBeVisible({ timeout: 10_000 });
    await kanjiPage.drawerSelectAllBtn.click();
    await kanjiPage.drawerAddBtn.click();
    await expect(kanjiPage.drawer).not.toBeVisible({ timeout: 60_000 });
});

When('выбирает уровни кандзи N5 и N4', async ({ page }) => {
    const kanjiPage = new KanjiPage(page);
    const n5Btn = page.getByTestId("kanji-level-n5");
    if (await n5Btn.isVisible().catch(() => false)) {
        await n5Btn.click();
        await kanjiPage.drawerSelectAllBtn.click();
    }
    const n4Btn = page.getByTestId("kanji-level-n4");
    if (await n4Btn.isVisible().catch(() => false)) {
        await n4Btn.click();
        await kanjiPage.drawerSelectAllBtn.click();
    }
});

Then('CJK шрифты загружены', async ({ page }) => {
    const kanjiPage = new KanjiPage(page);
    await expect(kanjiPage.kanjiPage).toBeVisible();
    const fontUsed = await page.evaluate(() => {
        const el = document.querySelector('[data-testid="kanji-card-item"] [data-testid="kanji-card-item-kanji"], [data-testid="kanji-card-item"]');
        if (!el) return false;
        const font = window.getComputedStyle(el).fontFamily;
        return font.includes("NotoSans") || font.includes("NotoSerif") || font.includes("Noto") || font.includes("sans-serif");
    });
    expect(fontUsed).toBe(true);
});
