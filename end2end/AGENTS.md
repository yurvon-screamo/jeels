# AGENTS.md — Origa E2E Tests (`end2end/`)

Playwright E2E tests for Origa. TypeScript + `@playwright/test`.

## npm Scripts

```bash
npm test                     # All tests (headless)
npm run test:ui              # Playwright UI mode
npm run test:headed          # Visible browser
npm run test:debug           # Debug mode
npm run test:chrome          # Chromium only
npm run test:chrome:headed   # Chromium, visible browser
npm run test:chrome:ui       # Chromium, Playwright UI
npm run report               # HTML report (0.0.0.0:9323)
npm run bdd:gen              # Generate Playwright tests from .feature files
npm run test:bdd             # bddgen + run BDD project only
npm run typecheck            # TypeScript type check (tsc --noEmit)
```

## Structure

```text
end2end/
├── .env                                    # TrailBase URL, admin creds
├── config.ts                              # Reads .env, exports getTrailBaseUrl()
├── global-setup.ts                        # Starts TrailBase, sets admin password
├── global-teardown.ts                     # Kills TrailBase process
├── playwright.config.ts                   # Playwright config + webServer definitions
├── pages/ {base,login,...}.page.ts → index.ts
├── tests/                                 # *.spec.ts
├── fixtures/                              # Auth, onboarding, test data
└── helpers/                               # Navigation, auth, HTTP, cleanup
```

## Local Infrastructure

Playwright auto-manages all infrastructure — no manual setup needed.

| Service | Port | Managed by | Config |
|---------|------|-----------|--------|
| TrailBase | 4000 | `global-setup.ts` | Spawns `trail run --dev` |
| CDN (static files) | 8080 | `playwright.config.ts` webServer | `npx serve ../cdn` |
| App (trunk/wasm) | 1420 | `playwright.config.ts` webServer | `trunk serve` or `npx serve` |

**Just run:** `npx playwright test` — everything starts automatically.

**Required `.env`:**

```
ORIGA_ADMIN_EMAIL=admin@localhost
ORIGA_ADMIN_PASSWORD=secret
TRAILBASE_URL=http://127.0.0.1:4000
ORIGA_CDN_BASE_URL=http://localhost:8080
```

**Flow:**

1. `playwright.config.ts` webServer starts CDN (8080) and app (1420)
2. `global-setup.ts` kills stale TrailBase, sets admin password, starts fresh TrailBase (4000)
3. Tests run against all three services
4. `global-teardown.ts` kills TrailBase process

## Test Users

| User | Email | Password | Purpose |
|------|-------|----------|---------|
| Admin | `admin@localhost` | `secret` | API operations, user management |
| Manual test | `uwuwu@uwuwu.net` | `uwuwu` | Manual UI testing (pre-created) |
| E2E auto-created | `e2e-<ts>-<rand>@origa.local` | `e2e-test-password-123` | Created/deleted per test |

E2E tests use `setupTestUser()` (in `helpers/auth.ts`) which creates a fresh user via admin API before each test and cleans up after.

## Page Object Pattern

All tests **must** use Page Objects from `pages/`. Select via `[data-testid="..."]` only — never CSS classes or text.

```typescript
import { Page } from '@playwright/test';
import { TestUser } from '../config';
export class LoginPage {
  constructor(private page: Page) {}
  async login(user: TestUser) {
    await this.page.fill('[data-testid="username"]', user.username);
    await this.page.fill('[data-testid="password"]', user.password);
    await this.page.click('[data-testid="login-button"]');
  }
}
```

## BDD Tests (Gherkin)

Complex multi-step user journeys can be described in Russian Gherkin `.feature` files under `bdd/features/`. Step definitions live in `bdd/steps/` and reuse Page Objects.

### When to use Gherkin vs native Playwright

- **Gherkin**: multi-step flows spanning multiple pages (onboarding, lesson lifecycle, sync). The `.feature` file is a separable spec that can be reviewed before implementation.
- **Native Playwright** (`.spec.ts`): single-page tests, DOM/CSS assertions, technical checks. Keep using `testWithFreshUser` + Page Objects.

### Pipeline

1. Write `.feature` file in `bdd/features/` (Russian, `# language: ru`).
2. Write step definitions in `bdd/steps/` — import `{ Given, When, Then }` from `bdd/fixtures.ts`.
3. Run `npm run bdd:gen` to generate Playwright tests into `.features-gen/`.
4. Run `npm run test:bdd` (or `npx playwright test --project=bdd`).

**Before `npm test` (all projects)**: run `npm run bdd:gen` first — the BDD project expects generated files in `.features-gen/`.

### Conventions

- `.feature` language: Russian (`# language: ru`). Keywords: `Функция`, `Сценарий`, `Допустим` (Given), `Когда` (When), `Тогда` (Then), `И` (And).
- Domain terms in steps must match [`origa/docs/glossary.md`](../origa/docs/glossary.md).
- Step definitions reuse Page Objects from `pages/`. Do not use raw selectors in steps.
- The BDD fixture (`bdd/fixtures.ts`) provides the same authenticated `page` as `testWithFreshUser`.
- **When = action, Then = assertion.** Don't hide assertions in When steps. The `.feature` reader sees what's verified in the Then line.
- **Migration fidelity.** When converting `.spec.ts` → `.feature`, ALL original assertions must be preserved. Self-check against source.
- **Dead step detection.** After removing a step from `.feature`, grep for unconsumed definitions in `bdd/steps/`. `bddgen`/ESLint do NOT catch orphans.
- **Shared helpers.** Reusable test logic lives in `helpers/`. Both `.spec.ts` and `bdd/steps/*.ts` import from there — single source of truth.

## Rules

- Headless by default; use `:headed` / `:ui` scripts for debugging
- `async/await` everywhere — no raw Promises or `.then()`
- Web-first assertions: `await expect(locator).toBeVisible()`
- Tags: `test('name @smoke @critical', ...)`
- Run `npm run typecheck` before committing — ESLint does NOT catch undefined references in TypeScript. Only `tsc --noEmit` catches missing imports.

## Feature-flagged specs

`tests/acquaintance_flow.spec.ts` требует приложения, собранного с
флагом `acquaintance_mode`, и пропускается в дефолтном прогоне:

```bash
# терминал 1: приложение под флагом
cd origa_ui && ORIGA_CDN_BASE_URL="https://s3.origa.uwuwu.net" trunk serve --features csr,acquaintance_mode
# терминал 2: спека
ACQUAINTANCE_MODE=1 npx playwright test tests/acquaintance_flow.spec.ts
```

Включение прогона в CI — ask-first правка `.github/workflows/ci.yml`
(сборка второго webServer-профиля с флагом).
