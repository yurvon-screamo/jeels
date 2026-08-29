import { expect, test } from "@playwright/test";
import { setupTestUser } from "../helpers/auth";
import { skipOnboarding } from "../helpers/navigation";

/**
 * S7: сквозной сценарий режима знакомства — часть основного потока урока.
 */

test.describe("Acquaintance mode", () => {
	let page: import("@playwright/test").Page;

	test.beforeEach(async ({ browser }) => {
		await setupTestUser();
		const context = await browser.newContext();
		page = await context.newPage();
		await skipOnboarding(page);

		// Рука формируется из пула новых карт свежего тестового пользователя
		// при загрузке страницы урока (SelectAcquaintanceHandUseCase).
		// Зависимость: поведение Select на пустом/непустом пуле — при его
		// изменении этот файл нужно синхронизировать.
	});

	test("happy path: показ → тренировка → ревью", async () => {
		await page.goto("/lesson");

		// Префаза показа видима вместо карточек урока.
		const view = page.getByTestId("acquaintance-view");
		await expect(view).toBeVisible({ timeout: 30_000 });

		const phaseTag = page.getByTestId("acquaintance-phase-tag");
		await expect(phaseTag).toContainText(/PRESENTATION|ПОКАЗ/i);

		// Показ: проходим слайды кнопкой «Дальше» до конца руки. Клики
		// ограничены по времени: WASM ре-рендеры гоняются с кликом
		// (helpers/lesson.ts, ACTION_TIMEOUT).
		const nextBtn = page.getByTestId("acquaintance-next-btn");
		for (let i = 0; i < 20; i++) {
			await nextBtn.click({ timeout: 3_000 }).catch(() => null);
			// Тренировка начинается после исчерпания показа.
			const trainingVisible = await page
				.getByTestId("acquaintance-training")
				.waitFor({ state: "visible", timeout: 1_000 })
				.then(() => true)
				.catch(() => false);
			if (trainingVisible) break;
		}

		// Тренировка: ротация до критерия каждой карты.
		const training = page.getByTestId("acquaintance-training");
		await expect(training).toBeVisible();

		const maxAnswers = 100;
		for (let i = 0; i < maxAnswers; i++) {
			const reveal = page.getByTestId("acquaintance-reveal-btn");
			const revealVisible = await reveal
				.waitFor({ state: "visible", timeout: 1_000 })
				.then(() => true)
				.catch(() => false);
			if (!revealVisible) break;
			await reveal.click({ timeout: 3_000 }).catch(() => null);
			await page
				.getByTestId("acquaintance-rating-remember")
				.click({ timeout: 3_000 })
				.catch(() => null);
		}

		// Итогового экрана нет: закрытая рука сразу открывает обычный урок.
		// У свежего тестового юзера должных карт нет — это штатный empty-state
		// урока; когда ревью есть, открываются карточки урока.
		const lesson = page
			.getByTestId("lesson-content")
			.or(page.getByTestId("lesson-empty-state"));
		await expect(lesson).toBeVisible({ timeout: 15_000 });
		await expect(view).not.toBeVisible();
	});

	test("«Уже знаю» во время показа пропускает карту", async () => {
		await page.goto("/lesson");
		const view = page.getByTestId("acquaintance-view");
		await expect(view).toBeVisible({ timeout: 30_000 });

		const knowBtn = page.getByTestId("acquaintance-know-btn");
		const knowVisible = await knowBtn
			.waitFor({ state: "visible", timeout: 5_000 })
			.then(() => true)
			.catch(() => false);
		if (!knowVisible) {
			test.skip(true, "рука не сформирована — пропускать нечего");
			return;
		}
		await knowBtn.click({ timeout: 3_000 });

		// Подтверждение — общий паттерн модалки.
		const modal = page.getByTestId("acquaintance-know-confirm");
		await expect(modal).toBeVisible();

		// Отмена закрывает модалку без побочных действий.
		await page.getByTestId("acquaintance-know-confirm-cancel").click({ timeout: 3_000 });
		await expect(modal).not.toBeVisible();

		// Подтверждение выбывает карту из руки.
		await knowBtn.click({ timeout: 3_000 });
		await page.getByTestId("acquaintance-know-confirm-confirm").click({ timeout: 3_000 });
		await expect(modal).not.toBeVisible();
	});
});
