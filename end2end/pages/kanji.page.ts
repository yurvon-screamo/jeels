import { Page, Locator, expect } from "@playwright/test";
import { BasePage } from "./base.page";

type KanjiLevel = "N5" | "N4" | "N3" | "N2" | "N1";
type FilterType = "Все" | "Новые" | "Сложные" | "В процессе" | "Изученные";

export class KanjiPage extends BasePage {
    readonly kanjiPage: Locator;
    readonly kanjiCard: Locator;
    readonly kanjiTitle: Locator;
    readonly radicalsBtn: Locator;
    readonly backBtn: Locator;
    readonly addBtn: Locator;
    readonly searchInput: Locator;
    readonly kanjiGrid: Locator;    readonly firstKanjiCard: Locator;

    readonly emptyState: Locator;

    readonly drawer: Locator;
    readonly drawerAddBtn: Locator;
    readonly drawerSelectAllBtn: Locator;

    readonly deleteModal: Locator;
    readonly deleteConfirmBtn: Locator;
    readonly deleteCancelBtn: Locator;

    readonly detailPage: Locator;
    readonly loadMoreButton: Locator;

    readonly detailDeleteModal: Locator;
    readonly detailDeleteConfirmBtn: Locator;

    constructor(page: Page) {
        super(page);

        this.kanjiPage = page.getByTestId("kanji-page");
        this.kanjiCard = page.getByTestId("kanji-card");
        this.kanjiTitle = page.getByTestId("kanji-title");
        this.radicalsBtn = page.getByTestId("kanji-radicals-btn");
        this.backBtn = page.getByTestId("kanji-back-btn");
        this.addBtn = page.getByTestId("kanji-add-btn");
        this.searchInput = page.getByTestId("kanji-search-input");
        this.firstKanjiCard = page.getByTestId("kanji-card-item").first();
        this.kanjiGrid = page.getByTestId("kanji-grid");
        this.emptyState = page.getByTestId("kanji-empty-state");

        this.drawer = page.getByTestId("kanji-add-drawer");
        this.drawerAddBtn = page.getByTestId("kanji-drawer-add-btn");
        this.drawerSelectAllBtn = page.getByTestId("kanji-drawer-select-all-btn");

        this.deleteModal = page.getByTestId("kanji-delete-modal");
        this.deleteConfirmBtn = page.getByTestId("kanji-delete-modal-confirm");
        this.deleteCancelBtn = page.getByTestId("kanji-delete-modal-cancel");

        this.loadMoreButton = page.getByTestId("kanji-load-more-btn");

        this.detailPage = page.getByTestId("kanji-detail");

        this.detailDeleteModal = page.getByTestId("kanji-detail-delete-modal");
        this.detailDeleteConfirmBtn = page.getByTestId("kanji-detail-delete-modal-confirm");
    }

    async goto(): Promise<void> {
        await this.navigate("/kanji");
    }

    async expectKanjiVisible(): Promise<void> {
        await this.page.waitForURL(/\/kanji$/, { timeout: 10000 });
    }

    async searchKanji(query: string): Promise<void> {
        await this.searchInput.fill(query);
    }

    async clickBack(): Promise<void> {
        await this.backBtn.click();
    }

    async clickRadicals(): Promise<void> {
        await this.radicalsBtn.click();
    }

    async openAddModal(): Promise<void> {
        await this.addBtn.click();
        await expect(this.drawer).toBeVisible({ timeout: 5000 });
    }

    async selectLevel(level: KanjiLevel): Promise<void> {
        await this.page.getByTestId("kanji-level-" + level.toLowerCase()).click();
        const kanjiItem = this.drawer.getByTestId("kanji-drawer-item").first();
        const emptyMsg = this.drawer.getByTestId("kanji-drawer-empty");
        await expect(kanjiItem.or(emptyMsg)).toBeVisible({ timeout: 10_000 });
    }

    async selectKanji(kanji: string): Promise<void> {
        const item = this.drawer.getByTestId("kanji-drawer-item").filter({ hasText: kanji });
        await expect(item).toBeVisible({ timeout: 5000 });
        await item.click();
    }

    async selectAllKanji(): Promise<void> {
        await this.drawerSelectAllBtn.click();
    }

    async addSelectedKanji(): Promise<void> {
        await this.drawerAddBtn.click({ timeout: 5000 });
        await expect(this.drawer).not.toBeVisible({ timeout: 15_000 });
    }

    async expectDetailPageVisible(): Promise<void> {
        await this.page.waitForURL(/\/kanji\//, { timeout: 5_000 });
        await expect(this.detailPage).toBeVisible({ timeout: 5_000 });
    }

    async goBack(): Promise<void> {
        await this.page.goBack();
    }

    async selectFilter(name: FilterType): Promise<void> {
        const filterMap: Record<FilterType, string> = {
            "Все": "all",
            "Новые": "new",
            "Сложные": "hard",
            "В процессе": "in-progress",
            "Изученные": "learned",
        };
        await this.page.getByTestId("kanji-filter-" + filterMap[name]).click();
    }

    async getCardCount(): Promise<number> {
        return this.page.getByTestId("kanji-card-item").count();
    }

    async deleteCardByIndex(index: number): Promise<void> {
        await this.clickCardActionBtn(index, "kanji-card-item-delete-btn");
        await expect(this.deleteModal).toBeVisible({ timeout: 5000 });
        await this.deleteConfirmBtn.click();
        await expect(this.deleteModal).not.toBeVisible({ timeout: 10_000 });
    }

    async cancelDeleteCardByIndex(index: number): Promise<void> {
        await this.clickCardActionBtn(index, "kanji-card-item-delete-btn");
        await expect(this.deleteModal).toBeVisible({ timeout: 5000 });
        await this.deleteCancelBtn.click();
        await expect(this.deleteModal).not.toBeVisible({ timeout: 5000 });
    }

    async markCardAsKnownByIndex(index: number): Promise<void> {
        await this.clickCardActionBtn(index, "kanji-card-item-mark-known-btn");
    }

    // Buttons use stop_propagation() in the component, so a bubbling click
    // won't reach the parent card's on:click handler that navigates to detail page.
    // Leptos event delegation requires bubbles: true for the handler to fire.
    private async clickCardActionBtn(index: number, btnTestId: string): Promise<void> {
        const selector = `[data-testid="kanji-card-item"]:nth-of-type(${index + 1}) [data-testid="${btnTestId}"]`;
        await this.page.evaluate((sel: string) => {
            const el = document.querySelector(sel) as HTMLElement;
            if (el) {
                el.dispatchEvent(new MouseEvent("click", { bubbles: true }));
            }
        }, selector);
    }

    async isLoadMoreVisible(): Promise<boolean> {
        return this.loadMoreButton.isVisible();
    }

    async clickLoadMore(): Promise<void> {
        await this.loadMoreButton.click();
    }

    async getFavoriteButton(index: number): Promise<Locator> {
        const card = this.page.getByTestId("kanji-card-item").nth(index);
        return card.getByTestId("kanji-card-item-favorite-btn");
    }

    async isFavorited(index: number): Promise<boolean> {
        const btn = await this.getFavoriteButton(index);
        const filledPath = btn.locator('svg path[fill="currentColor"]');
        return filledPath.isVisible().catch(() => false);
    }

    async toggleFavoriteByIndex(index: number): Promise<void> {
        const card = this.page.getByTestId("kanji-card-item").nth(index);
        const btn = card.getByTestId("kanji-card-item-favorite-btn");
        await btn.dispatchEvent("click");
        await this.page.waitForTimeout(1000);
    }

    async deleteFromDetail(): Promise<void> {
        // Both desktop and mobile action bars render — use .first() for strict mode compatibility
        const deleteBtn = this.page
            .getByTestId("kanji-detail-actions-delete-btn")
            .or(this.page.getByTestId("kanji-detail-actions-mobile-delete-btn"))
            .first();
        await expect(deleteBtn).toBeVisible({ timeout: 10_000 });
        await deleteBtn.click();
        await expect(this.detailDeleteModal).toBeVisible({ timeout: 5_000 });
        await this.detailDeleteConfirmBtn.click();
        await this.page.waitForURL(/\/kanji$/, { timeout: 10_000 });
    }
}
