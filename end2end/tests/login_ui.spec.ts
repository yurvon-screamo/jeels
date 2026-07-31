import { test, expect } from "@playwright/test";
import { LoginPage } from "../pages";
import { setupTestUser } from "../helpers/auth";

test.describe("Login UI", () => {
	test("shows spinner and disables submit while logging in @smoke", async ({
		page,
	}) => {
		const user = await setupTestUser();
		try {
			// Hold the login request until the in-flight state has been
			// observed. Manual gate beats a fixed setTimeout: the test
			// controls exactly when the request is released, so the spinner
			// assertion cannot flake on timing.
			let releaseRequest: () => void = () => {};
			const gate = new Promise<void>((resolve) => {
				releaseRequest = resolve;
			});
			await page.route("**/api/auth/v1/login", async (route) => {
				await gate;
				await route.continue();
			});

			const loginPage = new LoginPage(page);
			await loginPage.goto();
			await loginPage.expandPasswordForm();
			await loginPage.fillEmail(user.email);
			await loginPage.fillPassword(user.password);

			await loginPage.submit();
			await loginPage.expectSubmittingState();

			// Release the held request and confirm the full cycle completes.
			releaseRequest();
			await loginPage.expectLoginSuccess(["/home", "/onboarding"], 30_000);
		} finally {
			await user.cleanup();
		}
	});

	test("shows a divider between the header and the password section", async ({
		page,
	}) => {
		const loginPage = new LoginPage(page);
		await loginPage.goto();
		await loginPage.expectLoginFormVisible();

		// The divider sits between "Изучайте японский язык" (header) and the
		// "Войти с помощью пароля" toggle, mirroring the legal-links divider
		// at the bottom of the card.
		await expect(loginPage.headerDivider).toBeVisible();
	});
});
