import { Page, Locator, expect } from "@playwright/test";
import { BasePage } from "./base.page";

export class LoginPage extends BasePage {
    // Page structure
    readonly loginPage: Locator;
    readonly loginCard: Locator;

    // Form
    readonly loginForm: Locator;
    readonly emailInput: Locator;
    readonly passwordInput: Locator;
    readonly subtitle: Locator;
    readonly englishToggle: Locator;
    readonly passwordToggle: Locator;
    readonly passwordFormToggle: Locator;
    readonly submitButton: Locator;
    readonly errorAlert: Locator;

    // Layout
    readonly headerDivider: Locator;

    // Submit-state internals
    readonly spinner: Locator;

    // OAuth
    readonly appleButton: Locator;
    readonly googleButton: Locator;
    readonly yandexButton: Locator;

    constructor(page: Page) {
        super(page);

        // Page structure
        this.loginPage = page.getByTestId("login-page");
        this.loginCard = page.getByTestId("login-card");

        // i18n: the header subtitle renders login.subtitle localized
        // (RU «ИЗУЧАЙТЕ ЯПОНСКИЙ ЯЗЫК» / EN «Study Japanese»); the EN/RU
        // toggle is scoped to the login card so it never collides with the
        // profile page toggle of the same testid.
        this.subtitle = page.getByTestId("login-subtitle");
        this.englishToggle = page
            .getByTestId("login-lang-toggle")
            .getByTestId("lang-toggle-en");

        // Form
        this.loginForm = page.getByTestId("login-form");
        this.emailInput = page.getByTestId("email-input");
        this.passwordInput = page.getByTestId("password-input");
        this.passwordToggle = page.getByTestId("password-input-toggle");
        this.passwordFormToggle = page.getByTestId("login-password-toggle");
        this.submitButton = page.getByTestId("login-submit");
        this.errorAlert = page.getByTestId("login-form-error");

        // Layout
        this.headerDivider = page.getByTestId("login-header-divider");

        // The submit button renders an inline spinner (data-testid="btn-spinner"
        // in ui_components/button.rs) while a request is in flight. Scoped to
        // the submit button so it can't match a spinner on another control.
        this.spinner = this.submitButton.getByTestId("btn-spinner");

        // OAuth
        this.appleButton = page.getByTestId("oauth-apple");
        this.googleButton = page.getByTestId("oauth-google");
        this.yandexButton = page.getByTestId("oauth-yandex");
    }

    async goto(): Promise<void> {
        await this.navigate("/login");
    }

    /**
     * Reveal the email/password form. The form is collapsed by default behind
     * a "Sign in with password" toggle so the login card fits a mobile
     * viewport. Waits for the toggle to mount (races with WASM load) then
     * expands; no-op when already expanded.
     */
    async expandPasswordForm(): Promise<void> {
        // Idempotent per the contract above: when the inner form is already
        // open the toggle is unmounted (conditional render), so re-asserting
        // it would false-fail — return instead.
        const emailVisible = await this.emailInput.isVisible().catch(() => false);
        if (emailVisible) return;
        // The email/password form is collapsed behind the "Sign in with
        // password" toggle by default (mobile viewport fit). Wait for the
        // toggle to actually be ready, then click — guards against the race
        // where the page has just reloaded (e.g. after auth wipe) and the
        // WASM view hasn't mounted yet.
        const toggle = this.passwordFormToggle;
        await expect(toggle).toBeVisible({ timeout: 15_000 });
        await toggle.click();
        // Verify the inner form actually opened before returning, so callers
        // don't have to add their own wait.
        await expect(this.emailInput).toBeVisible({ timeout: 5_000 });
    }

    async expectLoginFormVisible(): Promise<void> {
        // Don't expand the password form here — that's the caller's
        // responsibility. Asserting on the inner inputs after expand belongs
        // in the explicit login-attempt step. We only verify the page-level
        // wrapper and the card are mounted.
        await expect(this.loginPage).toBeVisible();
        await expect(this.loginCard).toBeVisible();
    }

    async fillEmail(email: string): Promise<void> {
        await this.emailInput.waitFor({ state: "visible", timeout: 5000 });
        await this.emailInput.click({ force: true });
        await this.emailInput.fill(email, { force: true });
    }

    async fillPassword(password: string): Promise<void> {
        await this.passwordInput.waitFor({ state: "visible", timeout: 5000 });
        await this.passwordInput.click({ force: true });
        await this.passwordInput.fill(password, { force: true });
    }

    async togglePasswordVisibility(): Promise<void> {
        await this.passwordToggle.click();
    }

    async submit(): Promise<void> {
        await this.submitButton.waitFor({ state: "visible", timeout: 5000 });
        await this.submitButton.click({ force: true });
    }

    /**
     * Asserts the in-flight submit state: the submit button is disabled and a
     * spinner is rendered. Regression guard for the loader (it disappeared in
     * the collapsible-login refactor when the loading signal stopped being
     * threaded down to the form).
     */
    async expectSubmittingState(): Promise<void> {
        await expect(this.submitButton).toBeDisabled({ timeout: 5_000 });
        await expect(this.spinner).toBeVisible({ timeout: 5_000 });
    }

    async login(
        email: string,
        password: string,
    ): Promise<{ success: boolean; error?: string }> {
        try {
            await this.expandPasswordForm();
            await this.fillEmail(email);
            await this.fillPassword(password);
            await this.submit();
            await this.page.waitForURL(
                (url) => !url.pathname.includes("/login"),
                { timeout: 10_000 },
            );

            return { success: true };
        }
        catch (error) {
            return { success: false, error: error instanceof Error ? error.message : String(error) };
        }
    }

    async expectLoginSuccess(redirectTo: string | string[] = ["/home", "/onboarding"], timeout: number = 60000): Promise<void> {
        const paths = Array.isArray(redirectTo) ? redirectTo : [redirectTo];
        await this.page.waitForURL((url) => {
            const pathname = url.pathname;
            return paths.some(path => pathname.includes(path));
        }, { timeout });
    }

    async expectErrorMessage(): Promise<string | null> {
        await expect(this.errorAlert).toBeVisible();
        return await this.errorAlert.textContent();
    }

    async clickGoogleLogin(): Promise<void> {
        await this.googleButton.click();
    }

    async clickYandexLogin(): Promise<void> {
        await this.yandexButton.click();
    }
}
