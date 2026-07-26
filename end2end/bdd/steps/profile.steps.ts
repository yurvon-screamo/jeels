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
    await profilePage.langEnglish.click();
});

Then('отображается статус автосохранения', async ({ page }) => {
    const profilePage = new ProfilePage(page);
    await expect(profilePage.autosaveStatus).toBeVisible({ timeout: 10_000 });
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

Then('отображается карточка настроек с информацией о приложении', async ({ page }) => {
    const profilePage = new ProfilePage(page);
    await expect(profilePage.profileSettings).toBeVisible();
});

When('выбирает минимальную нагрузку', async ({ page }) => {
    const profilePage = new ProfilePage(page);
    await profilePage.loadMinimal.click();
});

Then('минимальная нагрузка выбрана', async ({ page }) => {
    const profilePage = new ProfilePage(page);
    await expect(profilePage.loadMinimal).toHaveClass(/selected|active/, { timeout: 10_000 });
});

Then('отображается подтверждение удаления', async ({ page }) => {
    await expect(page.getByTestId(/delete.*confirm|confirm.*delete/)).toBeVisible({ timeout: 5_000 });
});

When('пользователь отменяет удаление аккаунта', async ({ page }) => {
    const cancelBtn = page.getByTestId(/delete.*cancel|cancel.*delete/);
    await cancelBtn.click();
});

When('нажимает кнопку выхода', async ({ page }) => {
    await page.getByTestId("profile-logout").click();
});

Then('происходит переход на страницу входа', async ({ page }) => {
    await page.waitForURL(/\/login/, { timeout: 15_000 });
});

Then('отображается карточка смены пароля', async ({ page }) => {
    await expect(page.getByTestId("profile-password-change")).toBeVisible({ timeout: 10_000 });
});

When('вводит старый пароль {string}', async ({ page }, password: string) => {
    await page.getByTestId("password-change-current").fill(password);
});

When('вводит новый пароль {string}', async ({ page }, password: string) => {
    await page.getByTestId("password-change-new").fill(password);
});

When('вводит подтверждение {string}', async ({ page }, password: string) => {
    await page.getByTestId("password-change-confirm").fill(password);
});

When('нажимает кнопку смены пароля', async ({ page }) => {
    await page.getByTestId("password-change-submit").click();
});

Then('отображается ошибка несовпадения паролей', async ({ page }) => {
    await expect(page.getByTestId("password-change-error")).toBeVisible({ timeout: 10_000 });
});

When('нажимает кнопку удаления аккаунта', async ({ page }) => {
    await page.getByTestId("profile-delete-account").click();
});

When('подтверждает удаление', async ({ page }) => {
    await page.getByTestId("delete-account-confirm").click();
});

Then('отображается сообщение об успешной смене пароля', async ({ page }) => {
    await expect(page.getByTestId("password-change-success")).toBeVisible({ timeout: 10_000 });
});

Then('английский язык выбран', async ({ page }) => {
    const profilePage = new ProfilePage(page);
    await expect(profilePage.langEnglish).toHaveClass(/selected|active/, { timeout: 10_000 });
});
