import { expect } from "@playwright/test";
import { Given, When, Then } from "../fixtures";
import { LoginPage } from "../../pages";

Given('пользователь вышел из аккаунта', async ({ page }) => {
    // The BDD fixture logs the user in and stores the TrailBase auth token in
    // localStorage, plus user/card data in IndexedDB. Clearing cookies alone
    // is not enough — Leptos rehydrates the session from these stores on the
    // next load, keeping the user authenticated. Wipe ALL client-side state
    // (cookies, localStorage, sessionStorage, IndexedDB, cache storage) so
    // the app falls back to the public /login route.
    await page.context().clearCookies();
    await page.goto("/");
    await page.evaluate(async () => {
        try {
            window.localStorage.clear();
            window.sessionStorage.clear();
            if (window.indexedDB && indexedDB.databases) {
                const dbs = await indexedDB.databases();
                for (const db of dbs) {
                    if (db.name) indexedDB.deleteDatabase(db.name);
                }
              }
            if (window.caches) {
                const keys = await caches.keys();
                for (const k of keys) await caches.delete(k);
              }
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
    // The login form is collapsed behind a toggle by default; expand it so
    // the inner inputs become visible for the subsequent field assertions.
    const loginPage = new LoginPage(page);
    await loginPage.expandPasswordForm();
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
    // Submit button lives inside the expanded form; if a prior step didn't
    // expand it (or it re-collapsed), expand now so the click lands on the
    // real button instead of timing out.
    await loginPage.expandPasswordForm();
    await loginPage.submit();
});

Then('отображается сообщение об ошибке', async ({ page }) => {
    const loginPage = new LoginPage(page);
    await expect(loginPage.errorAlert).toBeVisible({ timeout: 10_000 });
});

Then('отображается кнопка Google', async ({ page }) => {
    const loginPage = new LoginPage(page);
    await expect(loginPage.googleButton).toBeVisible({ timeout: 10_000 });
});

Then('отображается кнопка Apple', async ({ page }) => {
    const loginPage = new LoginPage(page);
    await expect(loginPage.appleButton).toBeVisible({ timeout: 10_000 });
});

Then('отображается кнопка Yandex', async ({ page }) => {
    const loginPage = new LoginPage(page);
    await expect(loginPage.yandexButton).toBeVisible({ timeout: 10_000 });
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
