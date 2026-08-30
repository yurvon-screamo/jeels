import { expect } from "@playwright/test";
import { When, Then, Given } from "../fixtures";
import { HomePage } from "../../pages";

// URL вынесен в константу: и локальный trunk-билд, и CI dist-билд
// компилируются с ORIGA_CDN_BASE_URL=http://localhost:8080 (end2end/.env
// и ci.yml e2e-build) — паттерн одинаков на обоих окружениях.
const CDN_URL = "http://localhost:8080/**";
const LOGIN_API = "**/api/auth/v1/login";

Given('CDN недоступен', async ({ page }) => {
    await page.context().route(CDN_URL, (route) => route.abort());
});

Then('каркас приложения отображается', async ({ page }) => {
    // Деградация без краша: карточка приложения отвечает, навигация
    // смонтирована (свежий юзер — пустое состояние главной).
    const homePage = new HomePage(page);
    await expect(homePage.sidebar).toBeVisible({ timeout: 30_000 });
});

// Бут при лежащем CDN блокируется экраном загрузки словарей (X из Y) —
// наблюдаемая деградация вместо чёрного экрана/краша.
Then('отображается экран загрузки словарей', async ({ page }) => {
    await expect(page.getByTestId("app-loading-overlay")).toBeVisible({
        timeout: 30_000,
    });
});

Given('сервер аутентификации недоступен', async ({ page }) => {
    await page.route(LOGIN_API, (route) => route.abort());
});

When('сервер аутентификации восстановился', async ({ page }) => {
    await page.unroute(LOGIN_API);
});
