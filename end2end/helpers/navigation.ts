import { expect, type Page } from "@playwright/test";

export async function skipOnboarding(page: Page): Promise<void> {
	await expect(page.getByTestId("onboarding-spinner")).not.toBeVisible({
		timeout: 30_000,
	});
	const skipButton = page.getByTestId("onboarding-skip");
	if (await skipButton.isVisible().catch(() => false)) {
		await skipButton.click();
		// Confirm the skip action in the modal dialog
		const confirmButton = page.getByTestId("onboarding-confirm-ok");
		if (await confirmButton.isVisible().catch(() => false)) {
			await confirmButton.click();
		}
	}
	await page.waitForURL(/\/home$/, { timeout: 30_000 });
}
