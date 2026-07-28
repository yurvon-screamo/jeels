import { expect } from "@playwright/test";
import { Given, When, Then } from "../fixtures";
import { GrammarPage, HomePage, LessonPage, WordsPage } from "../../pages";
import { rateCardUntilDone } from "../../helpers/lesson";

Given('у пользователя есть добавленная грамматическая карточка', async ({ page }) => {
    const grammarPage = new GrammarPage(page);
    await grammarPage.goto();
    await expect(grammarPage.grammarPage).toBeVisible({ timeout: 15_000 });
    await grammarPage.openAddModal();
    await grammarPage.selectRule("～ます");
    await grammarPage.addSelectedRules();
    await expect(grammarPage.grammarGrid).toBeVisible({ timeout: 30_000 });
});

// Grammar practice (quiz) pulls questions from the user's own vocabulary that
// matches the rule (e.g. ます needs verb vocabulary). Without a matching word
// the practice session renders a "no words" empty state and the quiz tests
// can't exercise the option/next/complete flow.
Given('у пользователя есть слово для практики грамматики', async ({ page }) => {
    const wordsPage = new WordsPage(page);
    await wordsPage.goto();
    await wordsPage.expectWordsVisible();
    await wordsPage.openAddModal();
    // "私は本を読みます" tokenizes to 私 / 本 / 読む. 読む (yomu, "to read") is a
    // godan verb — the only token that conjugates with the ます rule, so we
    // must pick it specifically rather than relying on selectFirstWord.
    // Furigana annotations break pure-kanji matching (the item shows as
    // "読(ヨ)む"), so match on the Russian translation gloss instead.
    await wordsPage.enterText("私は本を読みます");
    await wordsPage.analyzeText();

    const yomuItem = wordsPage.drawer.getByTestId("words-drawer-item").filter({ hasText: "читать" }).first();
    await expect(yomuItem).toBeVisible({ timeout: 10_000 });
    // analyze_text() pre-selects every token; click once to be sure the item
    // toggles to the desired state, then verify via its checkbox.
    await yomuItem.click();
    const yomuCheckbox = yomuItem.locator('input[type="checkbox"]');
    if (!(await yomuCheckbox.isChecked().catch(() => false))) {
        await yomuItem.click();
    }
    await expect(yomuCheckbox).toBeChecked({ timeout: 5_000 });

    await wordsPage.addSelectedWords();
    await expect(wordsPage.wordsGrid).toBeVisible({ timeout: 10_000 });

    // Practice session filters vocabulary by is_known_card || is_in_progress,
    // which excludes fresh "new" cards. Rate the card via a one-shot lesson so
    // it crosses into in_progress and shows up in quiz generation.
    const homePage = new HomePage(page);
    await homePage.goto();
    await homePage.startLesson();
    const lessonPage = new LessonPage(page);
    await expect(lessonPage.lessonPage).toBeVisible({ timeout: 15_000 });
    await expect(lessonPage.lessonLoading).toBeHidden({ timeout: 30_000 });
    await expect(lessonPage.lessonContent).toBeVisible({ timeout: 15_000 });
    await rateCardUntilDone(lessonPage, "good");
    await expect(lessonPage.completeScreen).toBeVisible({ timeout: 15_000 });
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
    // The "add" button is disabled until at least one rule is selected. If a
    // previous step (e.g. "выбирает уровни грамматики N5 и N4") already
    // selected rules via selectAll, just submit; otherwise pick the canonical
    // first rule ("～ます") so the step is self-sufficient.
    if (await grammarPage.drawerAddBtn.isEnabled().catch(() => false)) {
        await grammarPage.addSelectedRules();
    } else {
        await grammarPage.selectRule("～ます");
        await grammarPage.addSelectedRules();
    }
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
    // Other Given steps (e.g. word setup) may have left the user on /words;
    // make sure we are on the grammar list before drilling into a card.
    if (!page.url().includes("/grammar/")) {
        const grammarPage = new GrammarPage(page);
        await grammarPage.goto();
        await expect(grammarPage.grammarPage).toBeVisible({ timeout: 15_000 });
    }
    await page.getByTestId("grammar-card-item").first().click();
    await page.waitForURL(/\/grammar\//, { timeout: 10_000 });
});

Then('отображается страница деталей грамматики', async ({ page }) => {
    await page.waitForURL(/\/grammar\//, { timeout: 10_000 });
    await expect(page.getByTestId("grammar-detail-container")).toBeVisible({ timeout: 15_000 });
});

Then('отображается содержимое деталей грамматики', async ({ page }) => {
    await page.waitForURL(/\/grammar\//, { timeout: 10_000 });
    await expect(page.getByTestId("grammar-detail-container")).toBeVisible({ timeout: 15_000 });
    await expect(page.getByTestId("grammar-detail-breadcrumbs")).toBeVisible({ timeout: 5_000 });
});

When('нажимает хлебные крошки', async ({ page }) => {
    await page.getByTestId("grammar-detail-breadcrumbs-back").click();
    await page.waitForURL(/\/grammar$/, { timeout: 10_000 });
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
    // On desktop the practice session is rendered inline (no tabs to click);
    // on mobile it lives behind the "practice" tab. Pick whichever applies.
    const practiceTab = page.getByTestId("grammar-detail-tabs-practice");
    if (await practiceTab.isVisible().catch(() => false)) {
        await practiceTab.click();
    }
    await expect(
        page.getByTestId("grammar-practice-progress").or(page.getByTestId("grammar-practice-complete")).or(page.getByTestId("grammar-practice-no-words")),
    ).toBeVisible({ timeout: 15_000 });
});

Then('отображается сессия практики с вопросами', async ({ page }) => {
    await expect(page.getByTestId("grammar-practice-progress")).toBeVisible({ timeout: 15_000 });
});

Then('отображается вопрос практики', async ({ page }) => {
    await expect(page.getByTestId("grammar-practice-progress")).toBeVisible({ timeout: 15_000 });
});

Then('отображаются варианты ответа практики', async ({ page }) => {
    await expect(page.getByTestId("grammar-practice-option-0")).toBeVisible({ timeout: 10_000 });
});

When('отвечает на все вопросы практики', async ({ page }) => {
    // Each question: pick the first option, then advance via Next. After an
    // option is clicked the practice session locks all options (pointer-
    // events-none) and surfaces the Next button, so the loop has to prefer
    // Next over re-clicking an option. On the last question Next sets
    // is_completed and the completion screen replaces the question card.
    // Upper bound is QUESTION_COUNT (20 in Rust) + buffer for retries.
    const MAX_ANSWER_ITERATIONS = 30;
    for (let i = 0; i < MAX_ANSWER_ITERATIONS; i++) {
        const complete = page.getByTestId("grammar-practice-complete");
        if (await complete.isVisible({ timeout: 1_000 }).catch(() => false)) break;

        const nextBtn = page.getByTestId("grammar-practice-next-btn");
        if (await nextBtn.isVisible({ timeout: 500 }).catch(() => false)) {
            await nextBtn.click();
            continue;
        }

        const option = page.getByTestId("grammar-practice-option-0");
        if (await option.isVisible({ timeout: 2_000 }).catch(() => false)) {
            const klass = (await option.getAttribute("class")) ?? "";
            if (!klass.includes("pointer-events-none")) {
                await option.click();
                continue;
            }
        }
        await page.waitForTimeout(200);
    }
});

Then('отображается завершение практики', async ({ page }) => {
    await expect(page.getByTestId("grammar-practice-complete")).toBeVisible({ timeout: 15_000 });
});
