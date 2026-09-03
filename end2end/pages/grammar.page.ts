import { Page, Locator, expect } from "@playwright/test";
import { BasePage } from "./base.page";

type GrammarLevel = "N5" | "N4" | "N3" | "N2" | "N1";
type FilterType = "Все" | "Новые" | "Сложные" | "В процессе" | "Изученные";

export class GrammarPage extends BasePage {
    readonly grammarPage: Locator;
    readonly grammarCard: Locator;
    readonly grammarTitle: Locator;
    readonly backBtn: Locator;
    readonly addBtn: Locator;
    readonly searchInput: Locator;
    readonly grammarGrid: Locator;    readonly firstRuleCard: Locator;

    readonly emptyState: Locator;

    readonly drawer: Locator;
    readonly drawerAddBtn: Locator;
    readonly drawerSelectAllBtn: Locator;
    readonly drawerSearchInput: Locator;

    readonly deleteModal: Locator;
    readonly deleteConfirmBtn: Locator;
    readonly deleteCancelBtn: Locator;

    // Pagination
    readonly loadMoreButton: Locator;

    // Practice mode
    readonly practiceSession: Locator;
    readonly practiceProgress: Locator;
    readonly practiceCorrectCount: Locator;
    readonly practiceOptions: readonly [Locator, Locator, Locator, Locator];
    readonly practiceNextBtn: Locator;
    readonly practiceComplete: Locator;
    readonly practiceAgainBtn: Locator;
    readonly practiceNoWords: Locator;

    // Detail page
    readonly detailContainer: Locator;
    readonly detailBreadcrumbs: Locator;
    readonly detailBreadcrumbsBack: Locator;
    readonly detailFsrs: Locator;
    readonly detailActions: Locator;
    readonly detailDeleteModal: Locator;
    readonly detailDeleteConfirmBtn: Locator;

    constructor(page: Page) {
        super(page);

        this.grammarPage = page.getByTestId("grammar-page");
        this.grammarCard = page.getByTestId("grammar-card");
        this.grammarTitle = page.getByTestId("grammar-title");
        this.backBtn = page.getByTestId("grammar-back-btn");
        this.addBtn = page.getByTestId("grammar-add-btn");
        this.searchInput = page.getByTestId("grammar-search-input");
        this.firstRuleCard = page.getByTestId("grammar-card-item").first();
        this.grammarGrid = page.getByTestId("grammar-grid");
        this.emptyState = page.getByTestId("grammar-empty-state");

        this.drawer = page.getByTestId("grammar-add-drawer");
        this.drawerAddBtn = page.getByTestId("grammar-drawer-add-btn");
        this.drawerSelectAllBtn = page.getByTestId("grammar-drawer-select-all-btn");
        this.drawerSearchInput = page.getByTestId("grammar-drawer-search-input");

        this.deleteModal = page.getByTestId("grammar-delete-modal");
        this.deleteConfirmBtn = page.getByTestId("grammar-delete-modal-confirm");
        this.deleteCancelBtn = page.getByTestId("grammar-delete-modal-cancel");

        // Pagination
        this.loadMoreButton = page.getByTestId("grammar-load-more-btn");

        // Practice mode
        this.practiceSession = page.getByTestId("grammar-practice-session");
        this.practiceProgress = page.getByTestId("grammar-practice-progress");
        this.practiceCorrectCount = page.getByTestId("grammar-practice-correct-count");
        this.practiceOptions = [
            page.getByTestId("grammar-practice-option-0"),
            page.getByTestId("grammar-practice-option-1"),
            page.getByTestId("grammar-practice-option-2"),
            page.getByTestId("grammar-practice-option-3"),
        ] as const;
        this.practiceNextBtn = page.getByTestId("grammar-practice-next-btn");
        this.practiceComplete = page.getByTestId("grammar-practice-complete");
        this.practiceAgainBtn = page.getByTestId("grammar-practice-again-btn");
        this.practiceNoWords = page.getByTestId("grammar-practice-no-words");

        // Detail page
        this.detailContainer = page.getByTestId("grammar-detail-container");
        this.detailBreadcrumbs = page.getByTestId("grammar-detail-breadcrumbs");
        this.detailBreadcrumbsBack = page.getByTestId("grammar-detail-breadcrumbs-back");
        this.detailFsrs = page.getByTestId("grammar-detail-fsrs");
        this.detailActions = page.getByTestId("grammar-detail-actions");
        this.detailDeleteModal = page.getByTestId("grammar-detail-delete-modal");
        this.detailDeleteConfirmBtn = page.getByTestId("grammar-detail-delete-modal-confirm");
    }

    async goto(): Promise<void> {
        await this.navigate("/grammar");
    }

    async expectGrammarVisible(): Promise<void> {
        await this.page.waitForURL(/\/grammar$/, { timeout: 10000 });
    }

    async searchGrammar(query: string): Promise<void> {
        await this.searchInput.waitFor({ state: "visible", timeout: 5000 });
        await this.searchInput.fill(query);
    }

    async clickBack(): Promise<void> {
        await this.backBtn.click();
    }

    async openAddModal(): Promise<void> {
        await this.addBtn.click();
        await expect(this.drawer).toBeVisible({ timeout: 5000 });
    }

    async selectLevel(level: GrammarLevel): Promise<void> {
        await this.page.getByTestId("grammar-level-" + level.toLowerCase()).click();
        const ruleItem = this.drawer.getByTestId("grammar-drawer-item").first();
        const emptyMsg = this.drawer.getByTestId("grammar-drawer-empty");
        await expect(ruleItem.or(emptyMsg)).toBeVisible({ timeout: 10_000 });
    }

    async selectRule(title: string): Promise<void> {
        const rule = this.drawer.getByTestId("grammar-drawer-item").filter({ hasText: title });
        await expect(rule.first()).toBeVisible({ timeout: 5000 });
        await rule.first().click();
    }

    async selectAllRules(): Promise<void> {
        await this.drawerSelectAllBtn.click();
    }

    async addSelectedRules(): Promise<void> {
        await this.drawerAddBtn.click({ timeout: 5000 });
        await expect(this.drawer).not.toBeVisible({ timeout: 15_000 });
    }

    async navigateBackToList(): Promise<void> {
        if (this.page.url().match(/\/grammar\/.+/)) {
            await this.page.goBack();
            await this.page.waitForURL(/\/grammar$/, { timeout: 5000 });
        }
    }

    async selectFilter(name: FilterType): Promise<void> {
        await this.navigateBackToList();
        const filterMap: Record<FilterType, string> = {
            "Все": "all",
            "Новые": "new",
            "Сложные": "hard",
            "В процессе": "in-progress",
            "Изученные": "learned",
        };
        await this.page.getByTestId("grammar-filter-" + filterMap[name]).click();
    }

    async getCardCount(): Promise<number> {
        return this.page.getByTestId("grammar-card-item").count();
    }

    /**
     * Cards render asynchronously after the add-drawer closes (fetch +
     * WASM re-render): an instant count() races it and can read 0. Wait
     * for the first card, then assert there is more than one.
     */
    async expectMoreThanOneCard(): Promise<void> {
        await expect(
            this.page.getByTestId("grammar-card-item").first(),
        ).toBeVisible({ timeout: 30_000 });
        expect(await this.getCardCount()).toBeGreaterThan(1);
    }

    async deleteCardByIndex(index: number): Promise<void> {
        await this.clickCardActionBtn(index, "grammar-card-item-delete-btn");
        await expect(this.deleteModal).toBeVisible({ timeout: 5000 });
        await this.deleteConfirmBtn.click();
        await expect(this.deleteModal).not.toBeVisible({ timeout: 10_000 });
    }

    async cancelDeleteCardByIndex(index: number): Promise<void> {
        await this.clickCardActionBtn(index, "grammar-card-item-delete-btn");
        await expect(this.deleteModal).toBeVisible({ timeout: 5000 });
        await this.deleteCancelBtn.click();
        await expect(this.deleteModal).not.toBeVisible({ timeout: 5000 });
    }

    async markCardAsKnownByIndex(index: number): Promise<void> {
        await this.clickCardActionBtn(index, "grammar-card-item-mark-known-btn");
    }

    async getFavoriteButton(index: number): Promise<Locator> {
        const selector = `[data-testid="grammar-card-item"]:nth-of-type(${index + 1}) [data-testid="grammar-card-item-favorite-btn"]`;
        return this.page.locator(selector);
    }

    async isFavorited(index: number): Promise<boolean> {
        const btn = await this.getFavoriteButton(index);
        const filledPath = btn.locator('svg path[fill="currentColor"]');
        return filledPath.isVisible().catch(() => false);
    }

    async toggleFavoriteByIndex(index: number): Promise<void> {
        const card = this.page.getByTestId("grammar-card-item").nth(index);
        const btn = card.getByTestId("grammar-card-item-favorite-btn");
        await btn.dispatchEvent("click");
        await this.page.waitForTimeout(1000);
    }

    private async clickCardActionBtn(index: number, btnTestId: string): Promise<void> {
        const selector = `[data-testid="grammar-card-item"]:nth-of-type(${index + 1}) [data-testid="${btnTestId}"]`;
        await this.page.evaluate((sel: string) => {
            const el = document.querySelector(sel) as HTMLElement;
            if (el) {
                el.dispatchEvent(new MouseEvent("click", { bubbles: true }));
            }
        }, selector);
    }

    async openPracticeForCard(index: number): Promise<void> {
        const card = this.page.getByTestId("grammar-card-item").nth(index);
        await card.click();
        await this.page.waitForURL(/\/grammar\/.+/, { timeout: 5000 });
        await expect(this.detailContainer).toBeVisible({ timeout: 30_000 });
        // Inline practice session renders on the detail page for rules with a
        // format map (replaces the former modal opened via a button).
        await expect(this.practiceSession).toBeVisible({ timeout: 10_000 });
    }

    async navigateToDetail(index: number): Promise<void> {
        const card = this.page.getByTestId("grammar-card-item").nth(index);
        await card.click();
        await this.page.waitForURL(/\/grammar\/.+/, { timeout: 5000 });
    }

    async selectPracticeOption(index: number): Promise<void> {
        await this.practiceOptions[index].click();
    }

    async clickPracticeNext(): Promise<void> {
        await this.practiceNextBtn.click();
    }

    async isLoadMoreVisible(): Promise<boolean> {
        return this.loadMoreButton.isVisible();
    }

    async clickLoadMore(): Promise<void> {
        await this.loadMoreButton.click();
    }
}
