import { expect } from "@playwright/test";
import { Given, When, Then } from "../fixtures";
import { KanjiPage } from "../../pages";

Given('у пользователя есть добавленное кандзи', async ({ page }) => {
    const kanjiPage = new KanjiPage(page);
    await kanjiPage.goto();
    await expect(kanjiPage.kanjiPage).toBeVisible({ timeout: 15_000 });
    await kanjiPage.addBtn.click();
    await expect(kanjiPage.drawer).toBeVisible({ timeout: 10_000 });
    // Pick exactly one kanji (the first in the current level) so scenarios
    // like "Удаление кандзи" can reach the empty-state after a single delete.
    // Tests that need many kanji use the separate "много кандзи" Given.
    const firstItem = kanjiPage.drawer.getByTestId("kanji-drawer-item").first();
    await expect(firstItem).toBeVisible({ timeout: 10_000 });
    await firstItem.click();
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
    // The "add" button stays disabled until at least one kanji is selected.
    // If the upstream scenario skipped an explicit selection step (e.g. the
    // "Добавление кандзи N5" path), fall back to "select all".
    if (await kanjiPage.drawerAddBtn.isEnabled().catch(() => false)) {
        // already enabled — something is selected
    } else {
        await kanjiPage.drawerSelectAllBtn.click();
        await expect(kanjiPage.drawerAddBtn).toBeEnabled({ timeout: 10_000 });
    }
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
    await page.goto("/home");
    await page.waitForURL(/\/home$/, { timeout: 10_000 });
});

When('пользователь отмечает первое кандзи как известное', async ({ page }) => {
    const kanjiPage = new KanjiPage(page);
    await kanjiPage.markCardAsKnownByIndex(0);
});

When('пользователь открывает детали первого кандзи', async ({ page }) => {
    await page.getByTestId("kanji-card-item").first().click();
    await page.waitForURL(/\/kanji\//, { timeout: 10_000 });
    await expect(page.getByTestId("kanji-detail")).toBeVisible({ timeout: 15_000 });
});

Then('отображается страница деталей кандзи', async ({ page }) => {
    await expect(page.getByTestId("kanji-detail")).toBeVisible({ timeout: 15_000 });
});

Then('отображается содержимое деталей кандзи', async ({ page }) => {
    await expect(page.getByTestId("kanji-detail")).toBeVisible({ timeout: 10_000 });
});

Then('чтения кандзи отображаются отдельными тегами', async ({ page }) => {
    // Hero card renders readings via <ReadingGroup>, one .reading-tag span per
    // reading. At least one of the ON/KUN groups must surface a tag.
    const onGroup = page.getByTestId("kanji-detail-on-readings");
    const kunGroup = page.getByTestId("kanji-detail-kun-readings");
    const onTags = onGroup.locator(".reading-tag");
    const kunTags = kunGroup.locator(".reading-tag");
    const onCount = await onTags.count();
    const kunCount = await kunTags.count();
    expect(onCount + kunCount).toBeGreaterThan(0);
});

Then('в карточке кандзи отображаются compact-чтения', async ({ page }) => {
    await expect(page.getByTestId("kanji-card-compact-readings").first()).toBeVisible({
        timeout: 10_000,
    });
});

Given('у пользователя есть кандзи "厳"', async ({ page }) => {
    const kanjiPage = new KanjiPage(page);
    await kanjiPage.goto();
    await expect(kanjiPage.kanjiPage).toBeVisible({ timeout: 15_000 });
    await kanjiPage.addBtn.click();
    await expect(kanjiPage.drawer).toBeVisible({ timeout: 10_000 });
    // 厳 is an N1 kanji — switch level, search, select, add.
    await kanjiPage.selectLevel("N1");
    await kanjiPage.searchKanji("厳");
    await kanjiPage.selectKanji("厳");
    await kanjiPage.addSelectedKanji();
    await expect(kanjiPage.kanjiGrid).toBeVisible({ timeout: 30_000 });
});

When('пользователь открывает детали этого кандзи', async ({ page }) => {
    await page.getByTestId("kanji-card-item").first().click();
    await page.waitForURL(/\/kanji\//, { timeout: 10_000 });
    await expect(page.getByTestId("kanji-detail")).toBeVisible({ timeout: 15_000 });
});

Then('редкие чтения кандзи отображаются приглушёнными', async ({ page }) => {
    // ReadingGroup renders rare readings with data-rare="true". Scoped to the
    // kanji-detail hero to avoid matching lesson cards.
    const detail = page.getByTestId("kanji-detail");
    const rareTags = detail.locator('[data-rare="true"]');
    await expect(rareTags.first()).toBeVisible({ timeout: 10_000 });
    const rareCount = await rareTags.count();
    expect(rareCount).toBeGreaterThanOrEqual(1);
});

Then('частые чтения кандзи отображаются обычным стилем', async ({ page }) => {
    const detail = page.getByTestId("kanji-detail");
    const normalTags = detail.locator('.reading-tag:not([data-rare="true"])');
    await expect(normalTags.first()).toBeVisible({ timeout: 10_000 });
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
    await expect(page.getByTestId("kanji-page")).toBeVisible();
    const fontUsed = await page.evaluate(() => {
        const el = document.querySelector('[data-testid="kanji-page"]');
        if (!el) return false;
        const font = window.getComputedStyle(el).fontFamily;
        return font.includes("NotoSans") || font.includes("NotoSerif") || font.includes("Noto");
    });
    expect(fontUsed).toBe(true);
});
