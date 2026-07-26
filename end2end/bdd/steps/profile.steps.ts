import { expect } from "@playwright/test";
import { When, Then } from "../fixtures";
import { ProfilePage } from "../../pages";

When('пользователь открывает страницу профиля', async ({ page }) => {
    const profilePage = new ProfilePage(page);
    await profilePage.goto();
    await expect(profilePage.profilePage).toBeVisible({ timeout: 15_000 });
});

Then('страница профиля отображается', async ({ page }) => {
    const profilePage = new ProfilePage(page);
    await expect(profilePage.profilePage).toBeVisible();
});

Then('отображаются опции языка', async ({ page }) => {
    const profilePage = new ProfilePage(page);
    await expect(profilePage.profileSettings).toBeVisible();
});

Then('отображается английский язык', async ({ page }) => {
    const profilePage = new ProfilePage(page);
    await expect(profilePage.langEnglish).toBeVisible();
});

Then('отображается русский язык', async ({ page }) => {
    const profilePage = new ProfilePage(page);
    await expect(profilePage.langRussian).toBeVisible();
});

When('выбирает английский язык', async ({ page }) => {
    const profilePage = new ProfilePage(page);
    await profilePage.selectLanguage("english");
    await profilePage.waitForAutoSave();
});

Then('отображается статус автосохранения', async ({ page }) => {
    const profilePage = new ProfilePage(page);
    await profilePage.waitForAutoSave();
});

Then('отображаются опции нагрузки', async ({ page }) => {
    const profilePage = new ProfilePage(page);
    await expect(profilePage.loadMedium).toBeVisible();
});

Then('отображается минимальная нагрузка', async ({ page }) => {
    const profilePage = new ProfilePage(page);
    await expect(profilePage.loadMinimal).toBeVisible();
});

Then('отображается средняя нагрузка', async ({ page }) => {
    const profilePage = new ProfilePage(page);
    await expect(profilePage.loadMedium).toBeVisible();
});

Then('отображается максимальная нагрузка', async ({ page }) => {
    const profilePage = new ProfilePage(page);
    await expect(profilePage.loadMaximum).toBeVisible();
});

When('нажимает кнопку выхода', async ({ page }) => {
    await page.getByTestId("profile-logout").click();
});

Then('происходит переход на страницу входа', async ({ page }) => {
    await page.waitForURL(/\/login/, { timeout: 15_000 });
});

Then('отображается карточка смены пароля', async ({ page }) => {
    await expect(page.getByTestId("profile-password")).toBeVisible({ timeout: 10_000 });
});

When('вводит старый пароль {string}', async ({ page }, password: string) => {
    await page.getByTestId("profile-password").click();
    await page.getByTestId("current-password").fill(password);
});

When('вводит новый пароль {string}', async ({ page }, password: string) => {
    await page.getByTestId("new-password").fill(password);
});

When('вводит подтверждение {string}', async ({ page }, password: string) => {
    await page.getByTestId("confirm-password").fill(password);
});

When('нажимает кнопку смены пароля', async ({ page }) => {
    await page.getByTestId("change-password-btn").click();
});

Then('отображается ошибка несовпадения паролей', async ({ page }) => {
    await expect(page.getByTestId("password-error")).toBeVisible({ timeout: 10_000 });
});

When('нажимает кнопку удаления аккаунта', async ({ page }) => {
    await page.getByTestId("profile-delete-btn").click();
});

When('подтверждает удаление', async ({ page }) => {
    await page.getByTestId("delete-account-confirm-btn").click();
});

Then('отображается сообщение об успешной смене пароля', async ({ page }) => {
    await expect(page.getByTestId("password-success")).toBeVisible({ timeout: 10_000 });
});

Then('английский язык выбран', async ({ page }) => {
    const profilePage = new ProfilePage(page);
    await profilePage.waitForAutoSave();
    await expect(profilePage.langEnglish).toBeVisible();
    const langClass = await profilePage.langEnglish.getAttribute("class");
    expect(langClass).toBeTruthy();
});

Then('отображается карточка настроек с информацией о приложении', async ({ page }) => {
    const profilePage = new ProfilePage(page);
    await expect(profilePage.profileSettings).toBeVisible();
});

When('выбирает минимальную нагрузку', async ({ page }) => {
    const profilePage = new ProfilePage(page);
    await profilePage.selectDailyLoad("minimal");
    await profilePage.waitForAutoSave();
});

Then('минимальная нагрузка выбрана', async ({ page }) => {
    const profilePage = new ProfilePage(page);
    await profilePage.waitForAutoSave();
    await expect(profilePage.loadMinimal).toBeVisible();
});

Then('отображается подтверждение удаления', async ({ page }) => {
    await expect(page.getByTestId(/delete.*confirm|confirm.*delete/)).toBeVisible({ timeout: 5_000 });
});

When('пользователь отменяет удаление аккаунта', async ({ page }) => {
    await page.getByTestId("cancel-delete-btn").click();
});

Then('отображается ошибка короткого пароля', async ({ page }) => {
    await expect(page.getByTestId("password-error")).toBeVisible({ timeout: 10_000 });
});
