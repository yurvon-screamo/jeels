import { Page, Locator, expect } from "@playwright/test";
import { BasePage } from "./base.page";

type WordsFilterType = "Все" | "Новые" | "Сложные" | "В процессе" | "Изученные";

export class WordsPage extends BasePage {
    // Page structure
    readonly wordsPage: Locator;
    readonly wordsCard: Locator;
    readonly wordsTitle: Locator;

    // Navigation buttons
    readonly backButton: Locator;
    readonly setsButton: Locator;
    readonly addButton: Locator;

    // Search
    readonly searchInput: Locator;

    // Content
    readonly wordsGrid: Locator;    readonly firstWordCard: Locator;

    readonly emptyState: Locator;

    // Add-words drawer
    readonly drawer: Locator;
    readonly drawerTextarea: Locator;
    readonly drawerAnalyzeBtn: Locator;
    readonly drawerAddBtn: Locator;
    readonly drawerCancelBtn: Locator;
    readonly analyzedWordItems: Locator;
    readonly noResultsFeedback: Locator;

    // Anki import
    readonly ankiTab: Locator;
    readonly ankiDropZone: Locator;
    readonly ankiFileInput: Locator;
    readonly ankiFieldWord: Locator;
    readonly ankiFieldTranslation: Locator;
    readonly ankiNextBtn: Locator;
    readonly ankiBackBtn: Locator;
    readonly ankiImportBtn: Locator;
    readonly ankiCardCount: Locator;
    readonly ankiCardList: Locator;
    readonly ankiDone: Locator;
    readonly ankiError: Locator;
    readonly ankiRetryBtn: Locator;

    // Image/OCR import
    readonly imageTab: Locator;
    readonly imageFileInput: Locator;

    // Audio import
    readonly audioTab: Locator;
    readonly audioFileInput: Locator;

    // Delete modal
    readonly deleteModal: Locator;
    readonly deleteConfirmBtn: Locator;
    readonly deleteCancelBtn: Locator;

    // Pagination
    readonly loadMoreButton: Locator;

    constructor(page: Page) {
        super(page);

        // Page structure
        this.wordsPage = page.getByTestId("words-page");
        this.wordsCard = page.getByTestId("words-card");
        this.wordsTitle = page.getByTestId("words-title");

        // Navigation buttons
        this.backButton = page.getByTestId("words-back-btn");
        this.setsButton = page.getByTestId("words-sets-btn");
        this.addButton = page.getByTestId("words-add-btn");

        // Search
        this.searchInput = page.getByTestId("words-search-input");

        // Content
        this.firstWordCard = page.getByTestId("words-card-item").first();
        this.wordsGrid = page.getByTestId("words-grid");
        this.emptyState = page.getByTestId("words-empty-state");

        // Add-words drawer
        this.drawer = page.getByTestId("words-add-drawer");
        this.drawerTextarea = page.getByTestId("words-drawer-textarea");
        this.drawerAnalyzeBtn = page.getByTestId("words-drawer-analyze-btn");
        this.drawerAddBtn = page.getByTestId("words-drawer-add-btn");
        this.drawerCancelBtn = page.getByTestId("words-drawer-cancel-btn");
        this.analyzedWordItems = this.drawer.getByTestId("words-drawer-item");
        this.noResultsFeedback = this.drawer.getByTestId("words-no-results");

        // Anki import
        this.ankiTab = this.drawer.getByText("Anki");
        this.ankiDropZone = page.getByTestId("anki-import-drop-zone");
        this.ankiFileInput = page.getByTestId("anki-import-file-input");
        this.ankiFieldWord = page.getByTestId("anki-import-field-word");
        this.ankiFieldTranslation = page.getByTestId("anki-import-field-translation");
        this.ankiNextBtn = page.getByTestId("anki-import-next-btn");
        this.ankiBackBtn = page.getByTestId("anki-import-back-btn");
        this.ankiImportBtn = page.getByTestId("anki-import-import-btn");
        this.ankiCardCount = page.getByTestId("anki-import-card-count");
        this.ankiCardList = page.getByTestId("anki-import-card-list");
        this.ankiDone = page.getByTestId("anki-import-done");
        this.ankiError = page.getByTestId("anki-import-error");
        this.ankiRetryBtn = page.getByTestId("anki-import-retry-btn");

        // Image/OCR import
        this.imageTab = this.drawer.getByTestId("words-add-tabs-image");
        this.imageFileInput = this.drawer.locator('input[type="file"][accept*="image"]');

        // Audio import
        this.audioTab = this.drawer.getByTestId("words-add-tabs-audio");
        this.audioFileInput = this.drawer.locator('input[type="file"][accept*="audio"]');

        // Delete modal
        this.deleteModal = page.getByTestId("words-delete-modal");
        this.deleteConfirmBtn = page.getByTestId("words-delete-modal-confirm");
        this.deleteCancelBtn = page.getByTestId("words-delete-modal-cancel");

        // Pagination
        this.loadMoreButton = page.getByTestId("words-load-more-btn");
    }

    async goto(): Promise<void> {
        await this.navigate("/words");
    }

    async expectWordsVisible(): Promise<void> {
        await this.page.waitForURL(/\/words$/, { timeout: 10000 });
    }

    async searchWords(query: string): Promise<void> {
        await this.searchInput.fill(query);
    }

    async clickBack(): Promise<void> {
        await this.backButton.click();
    }

    async clickSets(): Promise<void> {
        await this.setsButton.click();
    }

    async openAddModal(): Promise<void> {
        await this.addButton.click();
        await expect(this.drawer).toBeVisible({ timeout: 5000 });
    }

    async enterText(text: string): Promise<void> {
        await this.drawerTextarea.waitFor({ state: "visible", timeout: 5000 });
        await this.drawerTextarea.fill(text);
    }

    async analyzeText(): Promise<void> {
        await this.drawerAnalyzeBtn.click();
        // Wait for analysis results - the "Найдено" text indicates completion
        await this.drawer.getByText(/Найдено/).waitFor({ state: "visible", timeout: 10_000 });
    }

    async analyzeTextNoResults(): Promise<void> {
        await this.drawerAnalyzeBtn.click();
        // When analysis returns 0 words, the "No words found" Alert with
        // testid "words-no-results" appears, plus the input controls remain
        // available (no dead-end).
        await expect(this.noResultsFeedback).toBeVisible({ timeout: 10_000 });
    }

    async selectFirstWord(): Promise<void> {
        const firstItem = this.analyzedWordItems.first();
        await firstItem.waitFor({ state: "visible", timeout: 5000 });

        // analyze_text() pre-selects every detected word. To pick ONLY the
        // first one, deselect all currently-checked items first, then click
        // the first to (re)select it.
        const count = await this.analyzedWordItems.count();
        for (let i = 0; i < count; i++) {
            const item = this.analyzedWordItems.nth(i);
            const checkbox = item.locator('input[type="checkbox"]');
            if (await checkbox.isChecked()) {
                // Skip disabled items (no translation found) — they can't be
                // toggled and will stay checked regardless.
                const isDisabled = await item.evaluate((el) =>
                    el.classList.contains("cursor-not-allowed"),
                ).catch(() => false);
                if (isDisabled) continue;

                await item.click();
                await expect(checkbox).not.toBeChecked({ timeout: 2000 });
            }
        }

        await firstItem.click();
        await expect(
            firstItem.locator('input[type="checkbox"]'),
        ).toBeChecked({ timeout: 2000 });
    }

    async selectAllAnalyzedWords(): Promise<void> {
        const count = await this.analyzedWordItems.count();
        for (let i = 0; i < count; i++) {
            const item = this.analyzedWordItems.nth(i);
            await item.waitFor({ state: "visible", timeout: 3000 });
            await item.click();
            await this.page.waitForTimeout(200);
        }
    }

    async markAllCardsAsKnown(): Promise<void> {
        const count = await this.getCardCount();
        for (let i = 0; i < count; i++) {
            const card = this.page.getByTestId("words-card-item").nth(i);
            const btn = card.getByTestId("words-card-item-mark-known-btn");
            if (await btn.isVisible().catch(() => false)) {
                await btn.click();
                await this.page.waitForTimeout(300);
            }
        }
    }

    async addSelectedWords(): Promise<void> {
        await this.drawerAddBtn.click({ timeout: 5000 });
        await expect(this.drawer).not.toBeVisible({ timeout: 15_000 });
    }

    async cancelAddModal(): Promise<void> {
        await this.drawerCancelBtn.click();
    }

    async switchToAnkiTab(): Promise<void> {
        await this.ankiTab.click();
    }

    async uploadAnkiFile(filePath: string): Promise<void> {
        await this.ankiFileInput.setInputFiles(filePath);
    }

    async switchToImageTab(): Promise<void> {
        await this.imageTab.click();
    }

    async uploadImageFile(filePath: string): Promise<void> {
        await this.imageFileInput.setInputFiles(filePath);
    }

    async switchToAudioTab(): Promise<void> {
        await this.audioTab.click();
    }

    async uploadAudioFile(filePath: string): Promise<void> {
        await this.audioFileInput.setInputFiles(filePath);
    }

    async selectFilter(name: WordsFilterType): Promise<void> {
        // Accept both Russian (UI labels) and English (feature-file style) keys.
        const filterMap: Record<string, string> = {
            "Все": "all",
            "all": "all",
            "Новые": "new",
            "new": "new",
            "Сложные": "hard",
            "hard": "hard",
            "В процессе": "in-progress",
            "in-progress": "in-progress",
            "in_progress": "in-progress",
            "learning": "in-progress",
            "Изученные": "learned",
            "learned": "learned",
        };
        const suffix = filterMap[name];
        if (!suffix) {
            throw new Error(`Unknown filter name: ${name}`);
        }
        await this.page.getByTestId("words-filter-" + suffix).click();
    }

    async getCardCount(): Promise<number> {
        return this.page.getByTestId("words-card-item").count();
    }

    async deleteCardByIndex(index: number): Promise<void> {
        const card = this.page.getByTestId("words-card-item").nth(index);
        await card.getByTestId("words-card-item-delete-btn").click();
        await expect(this.deleteModal).toBeVisible({ timeout: 5000 });
        await this.deleteConfirmBtn.click();
        await expect(this.deleteModal).not.toBeVisible({ timeout: 10_000 });
    }

    async cancelDeleteCardByIndex(index: number): Promise<void> {
        const card = this.page.getByTestId("words-card-item").nth(index);
        await card.getByTestId("words-card-item-delete-btn").click();
        await expect(this.deleteModal).toBeVisible({ timeout: 5000 });
        await this.deleteCancelBtn.click();
        await expect(this.deleteModal).not.toBeVisible({ timeout: 5000 });
    }

    async markCardAsKnownByIndex(index: number): Promise<void> {
        const card = this.page.getByTestId("words-card-item").nth(index);
        await card.getByTestId("words-card-item-mark-known-btn").click();
    }

    async isLoadMoreVisible(): Promise<boolean> {
        return this.loadMoreButton.isVisible();
    }

    async clickLoadMore(): Promise<void> {
        await this.loadMoreButton.click();
    }

    async getFavoriteButton(index: number): Promise<Locator> {
        const card = this.page.getByTestId("words-card-item").nth(index);
        return card.getByTestId("words-card-item-favorite-btn");
    }

    async isFavorited(index: number): Promise<boolean> {
        const btn = await this.getFavoriteButton(index);
        const filledPath = btn.locator('svg path[fill="currentColor"]');
        return filledPath.isVisible().catch(() => false);
    }

    async toggleFavoriteByIndex(index: number): Promise<void> {
        const card = this.page.getByTestId("words-card-item").nth(index);
        const btn = card.getByTestId("words-card-item-favorite-btn");
        await btn.dispatchEvent("click");
        await this.page.waitForTimeout(500);
    }
}
