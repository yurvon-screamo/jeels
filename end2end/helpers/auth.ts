import { type Browser, type Page } from "@playwright/test";
import {
	getAdminToken,
	createTestUser,
	deleteTestUserWithRetry,
} from "../fixtures/admin";

export const DEFAULT_TEST_PASSWORD = "e2e-test-password-123";

export function generateUniqueEmail(): string {
	const timestamp = Date.now();
	const random = Math.random().toString(36).substring(2, 8);
	return `e2e-${timestamp}-${random}@origa.local`;
}

export interface TestUserContext {
	email: string;
	password: string;
	userUuid: string;
	adminToken: string;
	adminCsrfToken: string;
	cleanup: () => Promise<void>;
}

export async function setupTestUser(options?: {
	email?: string;
	password?: string;
}): Promise<TestUserContext> {
	const email = options?.email ?? generateUniqueEmail();
	const password = options?.password ?? DEFAULT_TEST_PASSWORD;

	const adminAuth = await getAdminToken();
	const userUuid = await createTestUser(
		adminAuth.token,
		adminAuth.csrfToken,
		email,
		password,
	);

	return {
		email,
		password,
		userUuid,
		adminToken: adminAuth.token,
		adminCsrfToken: adminAuth.csrfToken,
		cleanup: async () => {
			await deleteTestUserWithRetry(
				adminAuth.token,
				adminAuth.csrfToken,
				userUuid,
				email,
			);
		},
	};
}

export async function uiLogin(
	page: Page,
	email: string,
	password: string,
): Promise<void> {
	const maxRetries = 3;

	for (let attempt = 1; attempt <= maxRetries; attempt++) {
		// The whole login flow is retried, not just waitForURL: the password
		// toggle and the inputs race the WASM cold load too, and a toggle
		// miss used to escape the retry loop as an immediate failure.
		try {
			await page.goto("http://localhost:1420", {
				waitUntil: "domcontentloaded",
			});

			// The email/password form is collapsed behind a "Sign in with
			// password" toggle by default (mobile viewport fit). Wait for the
			// toggle to mount (races with WASM load) then expand it before
			// waiting for the inputs.
			const passwordToggle = page.getByTestId("login-password-toggle");
			await passwordToggle.waitFor({ state: "visible", timeout: 60_000 });
			await passwordToggle.click();

			await page
				.locator('input[type="email"], input[data-testid="email-input"]')
				.waitFor({ state: "visible", timeout: 30_000 });

			await page.fill(
				'input[type="email"], input[data-testid="email-input"]',
				email,
			);
			await page.fill(
				'input[type="password"], input[data-testid="password-input"]',
				password,
			);
			await page.click(
				'button[type="submit"], button[data-testid="login-submit"]',
			);

			await page.waitForURL(/\/(home|onboarding)/, { timeout: 60_000 });
			return;
		} catch (e) {
			if (attempt === maxRetries) {
				throw new Error(
					`Login failed after ${maxRetries} attempts for user ${email}: ${e}`,
				);
			}
			// brief cooldown before the retry — WASM may still be loading
			await page.waitForTimeout(3_000);
		}
	}
}

export async function withAuthenticatedPage(
	browser: Browser,
	use: (page: Page) => Promise<void>,
): Promise<void> {
	const context = await browser.newContext();
	const page = await context.newPage();
	await page.setViewportSize({ width: 1280, height: 720 });

	const userCtx = await setupTestUser();

	try {
		await uiLogin(page, userCtx.email, userCtx.password);
		await use(page);
	} finally {
		await context.close();
		await userCtx.cleanup();
	}
}
