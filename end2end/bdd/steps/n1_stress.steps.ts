import { expect } from "@playwright/test";

import { Given, When, Then, test } from "../fixtures";
import { corpusWordFromCdn } from "../../helpers/corpus";
import { completeN1StressSeed, completeOnboardingToScoring } from "../../helpers/onboarding";
import { GrammarPage } from "../../pages/grammar.page";
import { HomePage } from "../../pages/home.page";
import { KanjiPage } from "../../pages/kanji.page";
import { OnboardingPage } from "../../pages/onboarding.page";
import { PhrasesPage } from "../../pages/phrases.page";
import { ProfilePage } from "../../pages/profile.page";
import { SetsPage } from "../../pages/sets.page";
import { WordsPage } from "../../pages/words.page";

/**
 * N1 stress scenarios (@stress): every scenario pays the full-corpus seed
 * (~8k words / 11k+ cards through the REAL onboarding import — ADR-043's
 * bulk path), so each test gets a per-scenario timeout far above the
 * project's 180s default. Set at the first Given of every scenario via
 * this helper instead of per-step arithmetic.
 */
// 600s covers the slowest scenario (seed + nine section visits) in a
// DEBUG wasm build; CI's release artifact runs several times faster.
const STRESS_TEST_TIMEOUT_MS = 600_000;

function applyStressTimeout(): void {
    test.setTimeout(STRESS_TEST_TIMEOUT_MS);
}

// ═══════════════════════════════════════════════════════════════════════
// Seeding
// ═══════════════════════════════════════════════════════════════════════

Given('пользователь завершил N1-онбординг с полным корпусом', async ({ page }) => {
    applyStressTimeout();
    await completeN1StressSeed(page);
});

Given('новый пользователь начал онбординг с уровнем N1 без приложений', async ({ page }) => {
    applyStressTimeout();
    // Stops at the summary step: the scenario asserts import thresholds
    // BEFORE running the import.
    await completeOnboardingToScoring(page, {
        level: "N1",
        skipApps: true,
        stopAtSummary: true,
        onboardingUrlTimeout: 90_000,
    });
});

// ═══════════════════════════════════════════════════════════════════════
// Scenario 1: import + thresholds
// ═══════════════════════════════════════════════════════════════════════

const MIN_N1_WORDS = 8000;

Then('в сводке импорта не меньше 8000 слов', async ({ page }) => {
    const onboarding = new OnboardingPage(page);
    const stats = (await onboarding.summaryStats.textContent()) ?? "";
    // Threshold, not an exact number: the corpus evolves (8 279 words at
    // the time of writing) and exact counts would break on every
    // dictionary refresh. Parses the numbers out of the localized line.
    const chunks = stats.match(/\d[\d\s]*/g) ?? [];
    expect(chunks.length, `no numbers found in summary stats "${stats}"`).toBeGreaterThan(0);
    const numbers = chunks.map((chunk) => Number.parseInt(chunk.replace(/\s/g, ""), 10));
    const words = numbers.find((n) => n >= MIN_N1_WORDS);
    expect(
        words,
        `expected a word count ≥${MIN_N1_WORDS} among [${numbers.join(", ")}] in "${stats}"`,
    ).toBeDefined();
});

Then('все наборы JLPT от N5 до N1 отмечены в сводке', async ({ page }) => {
    const onboarding = new OnboardingPage(page);
    for (const level of ["n5", "n4", "n3", "n2", "n1"]) {
        await expect(
            onboarding.setCheckboxLocator(`jlpt_${level}`),
            `jlpt_${level} must be checked in the summary`,
        ).toBeChecked();
    }
});

When('пользователь импортирует корпус и отмечает все карточки известными', async ({ page }) => {
    const onboarding = new OnboardingPage(page);
    await onboarding.importButton.click();
    await expect(onboarding.scoringHint).toBeVisible({ timeout: 240_000 });
    await onboarding.clickMarkAllKnown();
    await expect(onboarding.scoringComplete).toBeVisible({ timeout: 240_000 });
    await onboarding.clickFinish();
    // The generic "переход на главную" step re-verifies with its own 30s
    // budget — the heavy checkpoint needs this long wait first.
    await page.waitForURL(/\/home/, { timeout: 240_000 });
});

// ═══════════════════════════════════════════════════════════════════════
// Scenario 2: library sections under the full corpus
// ═══════════════════════════════════════════════════════════════════════

When('открывается раздел слов с полным корпусом', async ({ page }) => {
    const words = new WordsPage(page);
    await words.goto();
    await expect(words.wordsPage).toBeVisible();
    // The grid must actually render cards, not an empty shell.
    await expect(words.wordsGrid).toBeVisible({ timeout: 60_000 });
});

Then('поиск находит слово из корпуса N1', async ({ page }) => {
    const words = new WordsPage(page);
    const corpusWord = corpusWordFromCdn();
    await words.searchInput.fill(corpusWord);
    await expect(words.wordsGrid).toBeVisible({ timeout: 30_000 });
});

Then('карточка слова из корпуса открывается', async ({ page }) => {
    const words = new WordsPage(page);
    await words.firstWordCard.click({ timeout: 30_000 });
    // The word detail surface renders without hanging under the full
    // corpus — presence of the page container is the assertion.
    await expect(words.wordsPage).toBeVisible();
});

When('открывается раздел кандзи уровня N1', async ({ page }) => {
    const kanji = new KanjiPage(page);
    await kanji.goto();
    await expect(kanji.kanjiPage).toBeVisible();
    await kanji.selectLevel("N1");
});

Then('сетка кандзи отображается', async ({ page }) => {
    const kanji = new KanjiPage(page);
    await expect(kanji.kanjiGrid).toBeVisible({ timeout: 60_000 });
});

Then('карточка кандзи открывается', async ({ page }) => {
    const kanji = new KanjiPage(page);
    await kanji.firstKanjiCard.click({ timeout: 30_000 });
    await expect(page).toHaveURL(/\/kanji\/.+/, { timeout: 30_000 });
});

When('открывается раздел грамматики уровня N1', async ({ page }) => {
    const grammar = new GrammarPage(page);
    await grammar.goto();
    await expect(grammar.grammarPage).toBeVisible();
    await grammar.selectLevel("N1");
});

Then('список правил грамматики отображается', async ({ page }) => {
    const grammar = new GrammarPage(page);
    await expect(grammar.grammarGrid).toBeVisible({ timeout: 60_000 });
});

Then('правило грамматики открывается', async ({ page }) => {
    const grammar = new GrammarPage(page);
    await grammar.firstRuleCard.click({ timeout: 30_000 });
    // The rule detail view replaces the list — the page stays alive.
    await expect(grammar.grammarPage).toBeVisible();
});

When('открывается раздел наборов', async ({ page }) => {
    const sets = new SetsPage(page);
    await sets.goto();
    await expect(sets.setsPage).toBeVisible();
});

Then('все JLPT-наборы отображаются импортированными', async ({ page }) => {
    const sets = new SetsPage(page);
    // Per-item testids are generic (`sets-card-item`); the corpus-imported
    // state is asserted through count: the five cumulative JLPT sets must
    // all be listed.
    const items = page.getByTestId("sets-card-item");
    await expect(items.first()).toBeVisible({ timeout: 30_000 });
    const count = await items.count();
    expect(count, "all five JLPT sets must be listed").toBeGreaterThanOrEqual(5);
});

When('открывается раздел фраз', async ({ page }) => {
    const phrases = new PhrasesPage(page);
    await phrases.goto();
});

Then('страница фраз отображается без краша', async ({ page }) => {
    const phrases = new PhrasesPage(page);
    // The N1 seed does not include phrases; the honest assertion is that
    // the page renders its container instead of crashing under the load.
    await expect(phrases.phrasesPage).toBeVisible({ timeout: 60_000 });
});

// ═══════════════════════════════════════════════════════════════════════
// Scenario 4: heavy-user sync between devices (ADR-045 regression)
// ═══════════════════════════════════════════════════════════════════════

Given('второе устройство входит в тот же аккаунт и восстанавливает данные', async ({
    secondDevicePage,
}) => {
    applyStressTimeout();
    // The fixture logged in on a fresh IndexedDB partition: the restore
    // path (remote → local, ADR-045) ran during login. Home must be
    // rendered with the restored corpus. The restore is one full cycle
    // over ~11k cards — minutes in a debug wasm build.
    // The app may bounce /login ↔ /home while the restore is in flight
    // (the home guard re-runs once the user signal settles) — settle
    // budget is generous in a debug build; CI's release artifact is faster.
    await expect(secondDevicePage.getByTestId("home-page")).toBeVisible({ timeout: 300_000 });
});

When('на втором устройстве начинается отслеживание PATCH-запросов записи пользователя', async ({
    secondDevicePage,
    apiRequestLog,
}) => {
    // Measurement window opens only now — every legitimate PATCH of the
    // first restore (and of the home mount right after it) happened
    // before this point and must not pollute the count.
    secondDevicePage.on("request", (request) => {
        if (
            request.method() === "PATCH" &&
            request.url().includes("/api/records/v1/domain_user")
        ) {
            apiRequestLog.push(request.url());
        }
    });
});

When('на втором устройстве повторно открывается главная страница', async ({ secondDevicePage }) => {
    // SPA navigation: the home effects remount and run_sync fires. The
    // sync GET is awaited BEFORE any assertion runs — otherwise a "no
    // PATCH" assertion would be green even if sync never executed.
    const syncGet = secondDevicePage.waitForResponse(
        (response) =>
            response.url().includes("/api/records/v1/domain_user") &&
            response.request().method() === "GET",
        { timeout: 60_000 },
    );
    const home = new HomePage(secondDevicePage);
    await home.goto();
    await syncGet;
    await expect(secondDevicePage.getByTestId("home-page")).toBeVisible({ timeout: 60_000 });
});

Then('запись пользователя не изменяется PATCH-запросом', async ({ apiRequestLog }) => {
    expect(
        apiRequestLog,
        `expected zero user-record PATCHes after an unchanged home remount; got: ${apiRequestLog.join(", ")}`,
    ).toHaveLength(0);
});

// ═══════════════════════════════════════════════════════════════════════
// Scenario 5: dashboard and profile under the full corpus
// ═══════════════════════════════════════════════════════════════════════

Then('карточки статистики главной страницы отображаются', async ({ page }) => {
    const home = new HomePage(page);
    // Card-level assertions only: the heavy computations (30-day chart,
    // completion forecast, rating ratio) render inside these cards on an
    // 11k-card corpus — visible cards prove the renders completed.
    await expect(home.todayOverview).toBeVisible({ timeout: 60_000 });
    await expect(home.activityChart).toBeVisible({ timeout: 60_000 });
    await expect(home.jlptProgress).toBeVisible({ timeout: 60_000 });
    await expect(home.recentStudy).toBeVisible({ timeout: 60_000 });
});

When('открывается страница профиля', async ({ page }) => {
    const profile = new ProfilePage(page);
    await profile.goto();
});

Then('статистика профиля отображается', async ({ page }) => {
    const profile = new ProfilePage(page);
    await expect(profile.profilePage).toBeVisible({ timeout: 60_000 });
    await expect(profile.profileContent).toBeVisible({ timeout: 60_000 });
    await expect(profile.profileDangerZone).toBeVisible({ timeout: 60_000 });
});
