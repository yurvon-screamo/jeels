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

// --- Миграция login_ui.spec.ts: спиннер/блокировка при входе ---

// Прямая навигация на /login (как в исходном спеке): после logout-вайпа
// приложение остаётся на «/», где форма входа живёт внутри ProtectedRoute
// и размонтируется глобальным syncing-оверлеем — спиннер-ассерт обязан
// проверяться на выделенном маршруте /login.
When('пользователь открывает страницу входа напрямую', async ({ page }) => {
    const loginPage = new LoginPage(page);
    await loginPage.goto();
    await loginPage.expectLoginFormVisible();
});

// Валидные креды текущего тестового пользователя (рандомные e2e-<ts>-<rand>@…):
// литеральные {string}-шаги выше годятся только для негативного логина.
When('пользователь вводит свой email', async ({ page, testUser }) => {
    const loginPage = new LoginPage(page);
    await loginPage.expandPasswordForm();
    await loginPage.fillEmail(testUser.email);
});

When('пользователь вводит свой пароль', async ({ page, testUser }) => {
    const loginPage = new LoginPage(page);
    await loginPage.fillPassword(testUser.password);
});

// Ручной гейт запроса входа: сервер «завис» — тест сам решает, когда
// отпустить. Фиксированный sleep здесь был бы флаком по определению.
When('запрос входа удерживается сервером', async ({ page, loginGateRelease }) => {
    let releaseRequest!: () => void;
    const gate = new Promise<void>((resolve) => {
        releaseRequest = resolve;
    });
    await page.route("**/api/auth/v1/login", async (route) => {
        await gate;
        await route.continue();
    });
    loginGateRelease.release = () => Promise.resolve(releaseRequest());
});

When('запрос входа отпущен сервером', async ({ loginGateRelease }) => {
    if (!loginGateRelease.release) {
        throw new Error("login request is not held — call the hold step first");
    }
    await loginGateRelease.release();
});

Then('кнопка входа заблокирована со спиннером', async ({ page }) => {
    const loginPage = new LoginPage(page);
    await loginPage.expectSubmittingState();
});

Then('вход завершается переходом в приложение', async ({ page }) => {
    const loginPage = new LoginPage(page);
    await loginPage.expectLoginSuccess(["/home", "/onboarding"], 30_000);
});

// --- Миграция login_ui.spec.ts: разделитель шапки ---

Then('отображается разделитель между шапкой и секцией пароля', async ({ page }) => {
    const loginPage = new LoginPage(page);
    await expect(loginPage.headerDivider).toBeVisible();
});

Then('отображается сообщение об ошибке', async ({ page }) => {
    const loginPage = new LoginPage(page);
    await expect(loginPage.errorAlert).toBeVisible({ timeout: 10_000 });
});

// --- Клавиатура: Enter в поле пароля отправляет форму ---

When('нажимает клавишу Enter в поле пароля', async ({ page }) => {
    const loginPage = new LoginPage(page);
    await loginPage.passwordInput.press("Enter");
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
