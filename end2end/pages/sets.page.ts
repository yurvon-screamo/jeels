import { Page, Locator, expect } from "@playwright/test";
import { BasePage } from "./base.page";

export class SetsPage extends BasePage {
    // Page structure
    readonly setsPage: Locator;
    readonly setsCard: Locator;
    readonly setsTitle: Locator;

    // Loading
    readonly loading: Locator;
    readonly loadingText: Locator;

    // Search
    readonly searchInput: Locator;

    // Level filters
    readonly levelAll: Locator;
    readonly levelN5: Locator;
    readonly levelN4: Locator;
    readonly levelN3: Locator;
    readonly levelN2: Locator;
    readonly levelN1: Locator;

    // Import filters
    readonly importAll: Locator;
    readonly importImported: Locator;
    readonly importNew: Locator;

    // Import actions
    readonly importSelectedBtn: Locator;
    readonly cancelSelectBtn: Locator;

    // Drawer (import preview modal)
    readonly drawer: Locator;
    readonly drawerImportBtn: Locator;
    readonly drawerCancelBtn: Locator;
    readonly drawerWordItems: Locator;

    // Pagination
    readonly loadMoreButton: Locator;

    constructor(page: Page) {
        super(page);

        // Page structure
        this.setsPage = page.getByTestId("sets-page");
        this.setsCard = page.getByTestId("sets-card");
        this.setsTitle = page.getByTestId("sets-title");

        // Loading
        this.loading = page.getByTestId("sets-loading");
        this.loadingText = page.getByTestId("sets-loading-text");

        // Search
        this.searchInput = page.getByTestId("sets-search-input");

        // Level filters
        this.levelAll = page.getByTestId("sets-level-all");
        this.levelN5 = page.getByTestId("sets-level-n5");
        this.levelN4 = page.getByTestId("sets-level-n4");
        this.levelN3 = page.getByTestId("sets-level-n3");
        this.levelN2 = page.getByTestId("sets-level-n2");
        this.levelN1 = page.getByTestId("sets-level-n1");

        // Import filters
        this.importAll = page.getByTestId("sets-import-all");
        this.importImported = page.getByTestId("sets-import-imported");
        this.importNew = page.getByTestId("sets-import-new");

        // Import actions
        this.importSelectedBtn = page.getByTestId("sets-import-selected-btn");
        this.cancelSelectBtn = page.getByTestId("sets-cancel-select-btn");

        // Drawer
        this.drawer = page.getByTestId("sets-import-drawer");
        this.drawerImportBtn = page.getByTestId("sets-drawer-import-btn");
        this.drawerCancelBtn = page.getByTestId("sets-drawer-cancel-btn");
        this.drawerWordItems = this.drawer.getByTestId("sets-drawer-item");

        // Pagination
        this.loadMoreButton = page.getByTestId("sets-load-more-btn");
    }

    async goto(): Promise<void> {
        await this.navigate("/sets");
    }

    async expectSetsVisible(): Promise<void> {
        await expect(this.setsPage).toBeVisible();
        await expect(this.setsCard).toBeVisible();
        await expect(this.setsTitle).toBeVisible();
    }

    async searchSets(query: string): Promise<void> {
        await this.searchInput.fill(query);
    }

    async selectLevelFilter(level: string): Promise<void> {
        await this.page.getByTestId(`sets-level-${level.toLowerCase()}`).click();
    }

    async selectTypeFilter(type: string): Promise<void> {
        await this.page.getByTestId(`sets-type-${type.toLowerCase()}`).click();
    }

    async selectImportFilter(filter: string): Promise<void> {
        await this.page.getByTestId(`sets-import-${filter.toLowerCase()}`).click();
    }

    async clickImportSelected(): Promise<void> {
        await this.importSelectedBtn.click();
    }

    async waitForLoading(): Promise<void> {
        await expect(this.loading).toBeVisible();
        await expect(this.loading).toBeHidden();
    }

    async waitForLoad(): Promise<void> {
        await this.searchInput.waitFor({ state: "visible", timeout: 60_000 });
    }

    async getSetCardCount(): Promise<number> {
        return this.page.getByTestId("sets-card-item").count();
    }

    async getImportedCardCount(): Promise<number> {
        // is_imported renders a reimport button (sets-card-reimport-btn)
        // instead of the regular import button. Counting those is more
        // robust than matching on the i18n-translated "Импортирован" tag,
        // which breaks if the test runner lands in an EN locale.
        return this.page.getByTestId("sets-card-reimport-btn").count();
    }

    getFirstNonImportedCard(): Locator {
        return this.page
            .getByTestId("sets-card-item")
            .filter({ has: this.page.getByTestId("sets-card-import-btn") })
            .first();
    }

    async clickImportOnCard(index: number): Promise<void> {
        const card = this.page.getByTestId("sets-card-item").nth(index);
        await card.getByTestId("sets-card-import-btn").click();
        await expect(this.drawer).toBeVisible({ timeout: 5_000 });
    }

    async importFromDrawer(): Promise<void> {
        await this.drawerImportBtn.click({ timeout: 5_000 });
        await expect(this.drawer).not.toBeVisible({ timeout: 30_000 });
    }

    async cancelImportFromDrawer(): Promise<void> {
        await this.drawerCancelBtn.click();
    }

    async selectSetCheckbox(index: number): Promise<void> {
        const card = this.page.getByTestId("sets-card-item").nth(index);
        await card.locator("label.checkbox-container").click();
    }

    async cancelSelection(): Promise<void> {
        await this.cancelSelectBtn.click();
    }

    async waitForDrawerWords(): Promise<void> {
        const foundText = this.drawer.getByTestId("sets-drawer-found");
        const emptyText = this.drawer.getByTestId("sets-drawer-empty");
        await expect(foundText.or(emptyText)).toBeVisible({ timeout: 15_000 });
    }

    async isLoadMoreVisible(): Promise<boolean> {
        return this.loadMoreButton.isVisible();
    }

    async clickLoadMore(): Promise<void> {
        await this.loadMoreButton.click();
    }
}
