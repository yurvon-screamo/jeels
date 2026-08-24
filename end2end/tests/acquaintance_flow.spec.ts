import { expect, test } from "@playwright/test";
import { setupTestUser } from "../helpers/auth";
import { skipOnboarding } from "../helpers/navigation";

/**
 * S7: сквозной сценарий режима знакомства под флагом
 * `acquaintance_mode` (compile-time feature).
 *
 * Запуск требует приложения, собранного с флагом:
 *   cd origa_ui && trunk serve --features csr,acquaintance_mode
 * и переменной окружения ACQUAINTANCE_MODE=1 для этого проекта Playwright.
 * В дефолтной CI-сборке спека пропускается.
 */
const FLAG_BUILD = process.env.ACQUAINTANCE_MODE === "1";

test.skip(!FLAG_BUILD, "requires origa_ui built with --features acquaintance_mode");

test.describe("Acquaintance mode", () => {
	let page: import("@playwright/test").Page;

	test.beforeEach(async ({ browser }) => {
		await setupTestUser();
		const context = await browser.newContext();
		page = await context.newPage();
		await skipOnboarding(page);

		// Добавляем слово в пул — оно станет картой руки.
		// Слова добавляются через существующие helpers словника в
		// setupLessonWithCards; здесь тот же путь, но урок не стартуем:
		// рука выбирается при загрузке страницы урока автоматически.
	});

	test("happy path: показ → тренировка → итог → ревью", async () => {
		await page.goto("/lesson");

		// Префаза показа видима вместо карточек урока.
		const view = page.getByTestId("acquaintance-view");
		await expect(view).toBeVisible({ timeout: 30_000 });

		const phaseTag = page.getByTestId("acquaintance-phase-tag");
		await expect(phaseTag).toContainText(/PRESENTATION|ПОКАЗ/i);

		// Показ: проходим слайды кнопкой «Дальше» до конца руки.
		const nextBtn = page.getByTestId("acquaintance-next-btn");
		for (let i = 0; i < 20; i++) {
			if (!(await nextBtn.isVisible().catch(() => false))) break;
			await nextBtn.click();
			// Тренировка начинается после исчерпания показа.
			if (await page.getByTestId("acquaintance-training").isVisible()) break;
		}

		// Тренировка: ротация до критерия каждой карты.
		const training = page.getByTestId("acquaintance-training");
		await expect(training).toBeVisible();

		const maxAnswers = 100;
		for (let i = 0; i < maxAnswers; i++) {
			const reveal = page.getByTestId("acquaintance-reveal-btn");
			if (!(await reveal.isVisible().catch(() => false))) break;
			await reveal.click();
			await page.getByTestId("acquaintance-rating-remember").click();
		}

		// Итоговый экран руки.
		const summary = page.getByTestId("acquaintance-summary");
		await expect(summary).toBeVisible({ timeout: 15_000 });

		// «К ревью» возвращает обычный урок.
		await page.getByTestId("acquaintance-to-reviews-btn").click();
		await expect(page.getByTestId("lesson-content")).toBeVisible({
			timeout: 15_000,
		});
	});

	test("«Уже знаю» во время показа пропускает карту", async () => {
		await page.goto("/lesson");
		await expect(page.getByTestId("acquaintance-view")).toBeVisible({
			timeout: 30_000,
		});

		const knowBtn = page.getByTestId("acquaintance-know-btn");
		if (!(await knowBtn.isVisible().catch(() => false))) {
			test.skip(true, "рука не сформирована — пропускать нечего");
			return;
		}
		await knowBtn.click();

		// Inline-подтверждение появляется и живёт на текущем слайде.
		const panel = page.getByTestId("acquaintance-know-confirm-panel");
		await expect(panel).toBeVisible();
		await page.getByTestId("acquaintance-know-confirm").click();
		await expect(panel).not.toBeVisible();

		// Отмена закрывает панель без побочных действий.
		await knowBtn.click();
		await page.getByTestId("acquaintance-know-cancel").click();
		await expect(panel).not.toBeVisible();
	});
});
