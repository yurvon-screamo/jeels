import { expect } from "@playwright/test";
import { Given, When, Then } from "../fixtures";
import { LoginPage } from "../../pages";

Given('пользователь вышел из аккаунта', async ({ page }) => {
    // The BDD fixture logs the user in and stores the TrailBase auth token in
    // localStorage. Clearing cookies alone is not enough — Leptos rehydrates
    // the session from localStorage on next load. Wipe all client-side state
    // so the app falls back to the public /login route.
    await page.context().clearCookies();
    await page.goto("/");
    await page.evaluate(() => {
        try {
            window.localStorage.clear();
            window.sessionStorage.clear();
        } catch {
            // ignore — some browsers block storage access in cross-origin iframes
        }
    });
    await page.reload();
});

When('открывается страница входа', async ({ page }) => {
    await page.waitForURL(/\/login|\/$/, { timeout: 15_000 });
    const loginPage = new LoginPage(page);
    await loginPage.expectLoginFormVisible();
});

Then('отображается форма входа', async ({ page }) => {
    const loginPage = new LoginPage(page);
    await expect(loginPage.loginForm).toBeVisible();
});

Then('отображается поле email', async ({ page }) => {
    const loginPage = new LoginPage(page);
    await expect(loginPage.emailInput).toBeVisible();
});

Then('отображается поле пароля', async ({ page }) => {
    const loginPage = new LoginPage(page);
    await expect(loginPage.passwordInput).toBeVisible();
});

Then('отображается кнопка входа', async ({ page }) => {
    const loginPage = new LoginPage(page);
    await expect(loginPage.submitButton).toBeVisible();
});

When('пользователь вводит email {string}', async ({ page }, email: string) => {
    const loginPage = new LoginPage(page);
    await loginPage.expandPasswordForm();
    await loginPage.fillEmail(email);
});

When('вводит пароль {string}', async ({ page }, password: string) => {
    const loginPage = new LoginPage(page);
    await loginPage.fillPassword(password);
});

When('нажимает кнопку входа', async ({ page }) => {
    const loginPage = new LoginPage(page);
    await loginPage.submit();
});

Then('отображается сообщение об ошибке', async ({ page }) => {
    const loginPage = new LoginPage(page);
    await expect(loginPage.errorAlert).toBeVisible({ timeout: 10_000 });
});

When('нажимает переключатель пароля', async ({ page }) => {
    const loginPage = new LoginPage(page);
    await loginPage.expandPasswordForm();
    await loginPage.passwordToggle.click();
});

Then('пароль становится видимым', async ({ page }) => {
    const loginPage = new LoginPage(page);
    await expect(loginPage.passwordInput).toHaveAttribute("type", "text");
});

Then('отображается переключатель языка', async ({ page }) => {
    await expect(page.getByTestId("lang-toggle-en").or(page.getByText("EN"))).toBeVisible();
});
