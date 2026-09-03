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
// CI (release artifact) measurements: dictionary load + full-corpus seed +
// the whole scenario run in ~2-3 minutes (run 33768068984: cold boot to the
// kanji phase in ~75s). The earlier "seed takes ~10 minutes" reading was an
// artifact: the kanji step used to await a drawer-only testid on the main
// page and simply burned the whole budget waiting for it. 600s is a 3x
// headroom for runner variance; a local DEBUG wasm build is slower and is
// not expected to fit — verify targeted steps locally, full runs belong
// to CI.
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
    // In-order matching survives the furigana ruby split of the card text
    // (see WordsPage.cardMatchingWord); .first() keeps the find-semantics
    // of the step ("the search finds the word") without an over-asserted
    // exact count. A no-op search is still caught probabilistically: the
    // grid renders the first 50 of the card_id-sorted cards, and a
    // mid-corpus word stays outside that batch under the current import
    // order.
    await expect(words.cardMatchingWord(corpusWord).first()).toBeVisible({
        timeout: 30_000,
    });
});

When('открывается раздел кандзи уровня N1', async ({ page }) => {
    const kanji = new KanjiPage(page);
    await kanji.goto();
    await expect(kanji.kanjiPage).toBeVisible();
    // The MAIN page filter (card_list_view), NOT the drawer's LevelSelector:
    // kanji-level-* only exists inside the add-kanji drawer.
    await kanji.selectJlptFilter("N1");
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

When('открывается раздел грамматики уровня N2', async ({ page }) => {
    const grammar = new GrammarPage(page);
    await grammar.goto();
    await expect(grammar.grammarPage).toBeVisible();
    // N2, not N1: the production grammar content (grammar.json) ships 515
    // rules covering N5–N2 and ZERO N1 rules — N1 grammar only exists in
    // the not-yet-merged grammar_v2.json (205 rules). Switch this back to
    // N1 when that content lands.
    await grammar.selectJlptFilter("N2");
});

Then('список правил грамматики уровня N2 отображается', async ({ page }) => {
    const grammar = new GrammarPage(page);
    // N2 ships 123 rules — the list must render a real subset of them
    // (progressive render: wait for the first item, then require a
    // meaningful subset).
    await expect(grammar.firstRuleCard).toBeVisible({ timeout: 60_000 });
    const ruleCount = await page.getByTestId("grammar-card-item").count();
    expect(
        ruleCount,
        `N2 grammar list must render at least 10 rules, got ${ruleCount}`,
    ).toBeGreaterThanOrEqual(10);
});

When('открывается раздел наборов', async ({ page }) => {
    const sets = new SetsPage(page);
    await sets.goto();
    await expect(sets.setsPage).toBeVisible();
});

Then('все JLPT-наборы отображаются импортированными', async ({ page }) => {
    const sets = new SetsPage(page);
    // Per-item testids are generic (`sets-card-item`); the corpus-imported
    // state is asserted through visibility: the five cumulative JLPT sets
    // must all be listed (progressive render — wait for the fifth).
    await expect(sets.setItemItems.first()).toBeVisible({ timeout: 30_000 });
    await expect(sets.setItemItems.nth(4)).toBeVisible({ timeout: 30_000 });
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
    // The measurement window opens only after the network goes quiet: the
    // first post-restore home mount runs its own sync (a late legitimate
    // PATCH is possible if the seed left the meta dirty), and that cycle
    // must complete BEFORE the window opens — otherwise a red here would
    // blame the skip path for seed-state noise (ADR-045).
    const hasDomainUserActivity = async (): Promise<boolean> => {
        const pending = apiRequestLog.length;
        await secondDevicePage.waitForTimeout(10_000);
        return apiRequestLog.length > pending;
    };

    // Every domain_user request (GET and PATCH alike) participates in the
    // quiet detection; the final assertion filters PATCH entries only.
    secondDevicePage.on("request", (request) => {
        if (request.url().includes("/api/records/v1/domain_user")) {
            apiRequestLog.push(`${request.method()} ${request.url()}`);
        }
    });

    for (let round = 0; round < 30; round++) {
        if (!(await hasDomainUserActivity())) {
            break;
        }
    }
    apiRequestLog.length = 0;
});

When('на втором устройстве повторно открывается главная страница', async ({
    secondDevicePage,
    apiRequestLog,
}) => {
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

    // The negative assertion needs the post-GET settle too: a broken skip
    // path decodes 11k cards and PATCHes seconds AFTER the GET response.
    // Quiet-window: 15s without new domain_user requests.
    let quiet = false;
    for (let round = 0; round < 12 && !quiet; round++) {
        const before = apiRequestLog.length;
        await secondDevicePage.waitForTimeout(5_000);
        quiet = apiRequestLog.length === before;
    }
});

Then('запись пользователя не изменяется PATCH-запросом', async ({ apiRequestLog }) => {
    // The log holds every domain_user request (GET and PATCH); the
    // contract counts only mutations.
    const patches = apiRequestLog.filter((entry) => entry.startsWith("PATCH "));
    expect(
        patches,
        `expected zero user-record PATCHes after an unchanged home remount; got: ${patches.join(", ")}`,
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
