import { expect } from "@playwright/test";
import { When, Then } from "../fixtures";
import { HomePage, LessonPage, WordsPage } from "../../pages";
import {
	answerTrainingForgot,
	answerTrainingRemember,
	awaitHandVisible,
	completeAcquaintanceHandIfPresent,
	completeLessonFlexible,
	completeTrainingUntilCriterion,
	runAcquaintancePresentation,
} from "../../helpers/lesson";
import { waitForUserRecordWrite } from "../../helpers/syncwait";

When('пользователь добавил слово из текста {string}', async ({ page }, text: string) => {
    const wordsPage = new WordsPage(page);
    await wordsPage.goto();
    await wordsPage.expectWordsVisible();
    await wordsPage.openAddModal();
    await wordsPage.enterText(text);
    await wordsPage.analyzeText();
    await wordsPage.selectFirstWord();
    await wordsPage.addSelectedWords();
    await expect(wordsPage.wordsGrid).toBeVisible({ timeout: 10_000 });
});

When('пользователь начинает урок', async ({ page }) => {
    const homePage = new HomePage(page);
    await homePage.goto();
    await homePage.startLesson();
});

When('пользователь добавил все слова из текста {string}', async ({ page }, text: string) => {
    const wordsPage = new WordsPage(page);
    await wordsPage.goto();
    await wordsPage.expectWordsVisible();
    await wordsPage.openAddModal();
    await wordsPage.enterText(text);
    await wordsPage.analyzeText();
    // analyze_text() pre-selects every token — add them all as-is.
    await wordsPage.addSelectedWords();
    await expect(wordsPage.wordsGrid).toBeVisible({ timeout: 10_000 });
});

When('пользователь отвечает в тренировке «Помню» {int} раз подряд', async ({ page, acqTrainingLog }, times: number) => {
    const training = page.getByTestId("acquaintance-training");
    await training.waitFor({ state: "visible", timeout: 10_000 });
    for (let i = 0; i < times; i++) {
        // Карта читается ДО ответа по свежему DOM (прошлый ответ подтверждён
        // ростом data-rotation-index внутри хелпера).
        acqTrainingLog.push((await training.getAttribute("data-card-id")) ?? "");
        const recorded = await answerTrainingRemember(page);
        expect(recorded, `ответ №${i + 1} не записался`).toBe(true);
    }
});

Then('каждый круг тренировки показывает каждую карту ровно один раз', async ({ acqTrainingLog }) => {
    const all = [...acqTrainingLog];
    const cards = Array.from(new Set(all));
    expect(cards.length, `в тренировке должно быть несколько карт, лог: ${all.join(",")}`).toBeGreaterThan(1);
    const fullRounds = Math.floor(all.length / cards.length);
    expect(fullRounds, `лог должен содержать хотя бы один полный круг, лог: ${all.join(",")}`).toBeGreaterThan(0);
    // Полные окна: множество карт каждого круга == множество всех карт,
    // без повторов внутри круга (независимо от перемешивания между кругами).
    for (let round = 0; round < fullRounds; round++) {
        const window = all.slice(round * cards.length, (round + 1) * cards.length);
        expect(
            new Set(window).size,
            `круг ${round + 1} не должен повторять карты, лог: ${all.join(",")}`,
        ).toBe(cards.length);
        expect(
            cards.every((c) => window.includes(c)),
            `круг ${round + 1} должен показать каждую карту, лог: ${all.join(",")}`,
        ).toBe(true);
    }
});

Then('направление тренировки всё ещё яп→рус', async ({ page }) => {
    await expect(page.getByTestId("acquaintance-direction-tag")).toContainText(
        /ЯП\s*→\s*РУС/,
    );
});

Then('направление тренировки меняется на рус→яп', async ({ page }) => {
    await expect(page.getByTestId("acquaintance-direction-tag")).toContainText(
        /РУС\s*→\s*ЯП/,
    );
});

Then('фронт тренировки показывает перевод', async ({ page }) => {
    const front = page.getByTestId("acquaintance-training-front");
    await front.waitFor({ state: "visible", timeout: 10_000 });
    const text = (await front.textContent()) ?? "";
    expect(
        text,
        "фронт рус→яп показывает перевод — без японских символов",
    ).not.toMatch(/[\u3040-\u30ff\u4e00-\u9faf]/);
});

Then('отображается страница урока с карточкой', async ({ page }) => {
    const lessonPage = new LessonPage(page);
    await expect(lessonPage.lessonPage).toBeVisible({ timeout: 15_000 });
    // Новый юзер начинает урок с руки знакомства — её слайд и есть карточка.
    const handVisible = await page
        .getByTestId("acquaintance-view")
        .waitFor({ state: "visible", timeout: 15_000 })
        .then(() => true)
        .catch(() => false);
    if (handVisible) return;
    await expect(lessonPage.lessonError).not.toBeVisible({ timeout: 15_000 });
    await expect(lessonPage.lessonLoading).toBeHidden({ timeout: 30_000 });
    await expect(lessonPage.lessonContent).toBeVisible({ timeout: 15_000 });
});

When('пользователь проходит руку знакомства', async ({ page }) => {
    const handSeen = await completeAcquaintanceHandIfPresent(page);
    expect(handSeen, "рука знакомства должна быть показана").toBe(true);
});

When('пользователь проходит показ руки знакомства', async ({ page }) => {
    const view = page.getByTestId("acquaintance-view");
    await expect(view).toBeVisible({ timeout: 15_000 });
    await runAcquaintancePresentation(page);
});

When('нажимает кнопку показа ответа тренировки', async ({ page }) => {
    const reveal = page.getByTestId("acquaintance-reveal-btn");
    const ready = await reveal
        .waitFor({ state: "visible", timeout: 20_000 })
        .then(() => true)
        .catch(() => false);
    if (!ready) {
        const trainingCount = await page
            .getByTestId("acquaintance-training")
            .count();
        const answerCount = await page
            .getByTestId("acquaintance-training-answer")
            .count();
        throw new Error(
            `reveal-btn not shown in 20s: training=${trainingCount}, answer=${answerCount}`,
        );
    }
    await reveal.click({ timeout: 5_000 });
});

Then('отображаются кнопки оценки тренировки', async ({ page }) => {
    await expect(
        page.getByTestId("acquaintance-rating-dont-know"),
    ).toBeVisible();
    await expect(page.getByTestId("acquaintance-rating-remember")).toBeVisible();
});

Then('отображается рука знакомства с карточкой', async ({ page }) => {
    await expect(page.getByTestId("acquaintance-view")).toBeVisible({
        timeout: 15_000,
    });
    await expect(page.getByTestId("acquaintance-phase-tag")).toContainText(
        /PRESENTATION|ПОКАЗ/i,
    );
});

Then('отображается полоса прогресса руки', async ({ page }) => {
    await expect(page.getByTestId("acquaintance-strip")).toBeVisible({
        timeout: 15_000,
    });
});

When('оценивает каждую карточку как Good', async ({ page }) => {
    const lessonPage = new LessonPage(page);
    // Новые карты идут через руку знакомства (acquaintance): проходим её
    // честно (показ → тренировка до критерия → переходный экран), после
    // чего добиваем классический урок рейтингом Good. Ревью карт руки
    // назначены на завтра — сразу после перехода урок обычно пуст.
    // Bounded wait (не мгновенный снапшот): рука может смонтироваться
    // позже проверки — иначе шаг молча ушёл бы в классическую ветку.
    if (await awaitHandVisible(page, 5_000)) {
        // Регистрируем ожидание ДО тренировки (как в sync.steps): сейв
        // закрытия руки (сидирование первого ревью + списание лимита)
        // летит на сервер из spawn_local в момент HandCompleted. Следующий
        // шаг шагает с полной перезагрузкой — сейв обязан landed раньше,
        // иначе рука переформируется из устаревшего состояния.
        const handSave = waitForUserRecordWrite(page, 20_000);
        await runAcquaintancePresentation(page);
        await completeTrainingUntilCriterion(page);
        const save = await handSave;
        expect(save.ok(), `hand close save failed: ${save.status()}`).toBe(true);
        await page.getByTestId("acquaintance-to-reviews-btn").click({ timeout: 5_000 });
    }
    if (await lessonPage.lessonContent.isVisible().catch(() => false)) {
        await completeLessonFlexible(lessonPage, page);
    }
});

When('нажимает кнопку возврата с урока', async ({ page }) => {
    const lessonPage = new LessonPage(page);
    await lessonPage.clickBack();
});

When('нажимает кнопку звука', async ({ page }) => {
    const lessonPage = new LessonPage(page);
    await lessonPage.toggleMute();
});

Then('звук переключён', async ({ page }) => {
    const lessonPage = new LessonPage(page);
    await expect(lessonPage.muteButton).toHaveAttribute("data-muted", "true");
});

When('пользователь устанавливает размер экрана планшета', async ({ page }) => {
    await page.setViewportSize({ width: 820, height: 1180 });
});

Then('содержимое урока занимает полную высоту', async ({ page }) => {
    const lessonPage = new LessonPage(page);
    await expect(lessonPage.lessonPage).toBeVisible({ timeout: 15_000 });
    await expect(lessonPage.lessonLoading).toBeHidden({ timeout: 30_000 });
    // Для нового юзера урок — это рука знакомства: слайд + панель действий.
    const handVisible = await page
        .getByTestId("acquaintance-view")
        .waitFor({ state: "visible", timeout: 15_000 })
        .then(() => true)
        .catch(() => false);
    if (handVisible) {
        const hand = page.getByTestId("acquaintance-view");
        const handHeight = await hand.evaluate(
            (el) => (el as HTMLElement).clientHeight,
        );
        expect(handHeight).toBeGreaterThan(500);
        return;
    }
    await expect(lessonPage.lessonContent).toBeVisible({ timeout: 15_000 });
    const height = await lessonPage.lessonContent.evaluate((el) => (el as HTMLElement).clientHeight);
    expect(height).toBeGreaterThan(700);
});

When('пользователь отмечает текущую карту руки известной', async ({ page, acqKnownCard }) => {
    const slide = page.getByTestId("acquaintance-slide");
    await slide.waitFor({ state: "visible", timeout: 15_000 });
    acqKnownCard.value = (await slide.getAttribute("data-card-id")) ?? null;
    await page.getByTestId("acquaintance-know-btn").click({ timeout: 5_000 });
    await page
        .getByTestId("acquaintance-know-confirm-confirm")
        .click({ timeout: 5_000 });
    await page
        .getByTestId("acquaintance-know-confirm")
        .waitFor({ state: "hidden", timeout: 15_000 })
        .catch(() => null);
});

Then('рука не уменьшается и показывается новая карта', async ({ page, acqKnownCard }) => {
    const slide = page.getByTestId("acquaintance-slide");
    await slide.waitFor({ state: "visible", timeout: 15_000 });
    // Замена асинхронна (mark-known → пул → слайды): ждём, пока слот
    // займёт новая карта — рендер мог не успеть за скрытием модалки.
    await expect
        .poll(async () => (await slide.getAttribute("data-card-id")) ?? "", {
            timeout: 15_000,
            intervals: [100, 200, 400],
        })
        .not.toBe(acqKnownCard ?? "");
    const aria = await page
        .getByTestId("acquaintance-strip")
        .getAttribute("aria-label");
    expect(aria, `рука не уменьшилась, aria: ${aria}`).toContain("1/");
    expect(Number((aria ?? "0/0").split("/")[1]), "размер руки сохранён").toBeGreaterThanOrEqual(7);
});

Then('отображается экран перехода к повторению', async ({ page }) => {
    await expect(page.getByTestId("acquaintance-completed")).toBeVisible({
        timeout: 15_000,
    });
    await expect(page.getByTestId("acquaintance-to-reviews-btn")).toBeVisible();
});

When('пользователь продолжает к повторению', async ({ page }) => {
    await page.getByTestId("acquaintance-to-reviews-btn").click({ timeout: 5_000 });
});

// --- Полный круг руки (миграция acquaintance_flow.spec.ts, S7) ---

// Тренировка до полного критерия — contrast с fast-path «Уже знаю»
// из шага «проходит руку знакомства»: честная ротация каждой карты.
When('пользователь отвечает в тренировке до полного критерия', async ({ page }) => {
    await completeTrainingUntilCriterion(page);
});

// После перехода к повторению у свежего юзера ревью-карт нет —
// открывается обычный урок либо его штатное пустое состояние.
Then('отображается содержимое урока или пустое состояние урока', async ({ page }) => {
    const lessonPage = new LessonPage(page);
    await expect(
        lessonPage.lessonContent.or(lessonPage.lessonEmptyState),
    ).toBeVisible({ timeout: 15_000 });
    await expect(page.getByTestId("acquaintance-view")).not.toBeVisible();
});

// Отмена подтверждения «Уже знаю»: модалка закрывается без побочных
// действий — карта не выбывает из руки и не заменяется. Видимость
// модалки ассертится отдельным Then (When = action, Then = assertion).
When('пользователь нажимает «Уже знаю» в показе', async ({ page, acqKnownCard }) => {
    const slide = page.getByTestId("acquaintance-slide");
    await slide.waitFor({ state: "visible", timeout: 15_000 });
    acqKnownCard.value = (await slide.getAttribute("data-card-id")) ?? null;
    await page.getByTestId("acquaintance-know-btn").click({ timeout: 5_000 });
});

Then('отображается подтверждение «Уже знаю»', async ({ page }) => {
    await expect(page.getByTestId("acquaintance-know-confirm")).toBeVisible();
});

When('пользователь отменяет подтверждение «Уже знаю»', async ({ page }) => {
    await page.getByTestId("acquaintance-know-confirm-cancel").click({ timeout: 5_000 });
});

Then('модалка закрыта и карта остаётся в руке', async ({ page, acqKnownCard }) => {
    await expect(page.getByTestId("acquaintance-know-confirm")).not.toBeVisible();
    const slide = page.getByTestId("acquaintance-slide");
    await expect(slide).toBeVisible();
    // Усиление сверх исходного спека: отменённое «Уже знаю» не должно
    // даже менять слот слайда — это та же карта, что была до клика.
    await expect
        .poll(async () => (await slide.getAttribute("data-card-id")) ?? "", {
            timeout: 5_000,
            intervals: [100, 200, 400],
        })
        .toBe(acqKnownCard.value ?? "");
    await expect(page.getByTestId("acquaintance-view")).toBeVisible();
});

// --- Клавиатура: Пробел на экране перехода продолжает к повторению
// (тот же хендл, что у кнопки — acquaintance_view.rs §8.3) ---

When('нажимает клавишу Пробел', async ({ page }) => {
    await page.keyboard.press(" ");
});

// --- Клавиатура в тренировке: Пробел до раскрытия = «Показать ответ»
// (acquaintance_keyboard.rs: Training + Space → Reveal). ---

Then('отображается ответ тренировки', async ({ page }) => {
    await expect(page.getByTestId("acquaintance-training-answer")).toBeVisible({
        timeout: 5_000,
    });
});

Then('последняя карта круга не открывает следующий', async ({ acqTrainingLog }) => {
    const all = [...acqTrainingLog];
    const cards = Array.from(new Set(all));
    const size = cards.length;
    expect(size).toBeGreaterThan(1);
    // Стыки кругов: последняя позиция круга N и первая круга N+1.
    for (let seam = size; seam + 1 < all.length; seam += size) {
        expect(
            all[seam] !== all[seam + 1],
            `стык после позиции ${seam + 1}: карта не должна идти дважды подряд`,
        ).toBe(true);
    }
});

When(
    'пользователь отвечает в тренировке «Помню» только по первой карте до закрытия',
    async ({ page, acqTargetCard }) => {
        const training = page.getByTestId("acquaintance-training");
        await training.waitFor({ state: "visible", timeout: 10_000 });
        acqTargetCard.value = (await training.getAttribute("data-card-id")) ?? null;
        // На целевой карте — «Помню», на остальных — «Не помню»: соседи
        // не должны закрыть Forward, иначе смена подфазы сбросит шкалы и
        // полоса потеряет закрытую карту ещё до ассерта.
        for (let i = 0; i < 40; i++) {
            const current = (await training.getAttribute("data-card-id")) ?? "";
            const recorded =
                current === acqTargetCard.value
                    ? await answerTrainingRemember(page)
                    : await answerTrainingForgot(page);
            expect(recorded, `ответ №${i + 1} не записался`).toBe(true);
            const closed = Number(
                (await page
                    .getByTestId("acquaintance-strip")
                    .getAttribute("aria-valuenow")) ?? "0",
            );
            if (closed >= 1) return;
        }
        throw new Error("первая карта не закрыла критерий за 40 ответов");
    },
);

Then('полоса руки показывает одну закрытую карту', async ({ page }) => {
    // aria-valuenow считает ячейки с заполнением текущей подфазы >= 3 —
    // ровно условие заморозки/переоткрытия карты.
    await expect
        .poll(
            async () =>
                Number(
                    (await page
                        .getByTestId("acquaintance-strip")
                        .getAttribute("aria-valuenow")) ?? "-1",
                ),
            { timeout: 15_000, intervals: [100, 200, 400] },
        )
        .toBe(1);
});

When(
    'пользователь отвечает в тренировке «Не помню» по закрытой карте',
    async ({ page, acqTargetCard }) => {
        const training = page.getByTestId("acquaintance-training");
        // Отвечаем «Не помню», пока круг не доходит до закрытой карты:
        // у остальных карт шкалы уже в нуле, ответ их не меняет.
        for (let i = 0; i < 20; i++) {
            const current = (await training.getAttribute("data-card-id")) ?? "";
            const recorded = await answerTrainingForgot(page);
            expect(recorded, `ответ №${i + 1} не записался`).toBe(true);
            if (current === acqTargetCard.value) return;
        }
        throw new Error("закрытая карта не встретилась в круге за 20 ответов");
    },
);

Then('полоса руки не показывает закрытых карт', async ({ page }) => {
    await expect
        .poll(
            async () =>
                Number(
                    (await page
                        .getByTestId("acquaintance-strip")
                        .getAttribute("aria-valuenow")) ?? "-1",
                ),
            { timeout: 15_000, intervals: [100, 200, 400] },
        )
        .toBe(0);
});
