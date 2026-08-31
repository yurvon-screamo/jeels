# Origa E2E Tests

Playwright E2E для Origa: TypeScript + `@playwright/test` + `playwright-bdd`
(Gherkin на русском). **Все сценарии — в Gherkin** (`bdd/features/*.feature`);
нативные `.spec.ts` декомиссированы, техмеханизмы (route-гейты, аборты CDN)
живут в шагах и `helpers/`.

## Запуск

```bash
npm ci                # зависимости
npm test              # всё (pretest сам гоняет bddgen)
npm run test:headed   # видимый браузер
npm run test:ui       # Playwright UI mode
npm run test:debug    # debug mode
npm run bdd:gen       # регенерация .features-gen из .feature
npm run test:bdd      # bddgen + только BDD-проект
npm run typecheck     # tsc --noEmit (обязателен перед коммитом)
npm run report        # HTML-отчёт (0.0.0.0:9323)
npm run guard:matrix  # проверка CI-матрицы (см. ниже)
```

Playwright сам поднимает всю инфраструктуру — вручную ничего не нужно:

| Сервис        | Порт | Кто поднимает            | Конфиг                |
|---------------|------|--------------------------|-----------------------|
| TrailBase     | 4000 | `global-setup.ts`        | `trail run --dev`     |
| CDN (статика) | 8080 | `playwright.config.ts`   | `npx serve ../cdn`    |
| Приложение    | 1420 | `playwright.config.ts`   | `trunk serve` / dist  |

`.env` (обязателен):

```
ORIGA_ADMIN_EMAIL=admin@localhost
ORIGA_ADMIN_PASSWORD=secret
TRAILBASE_URL=http://127.0.0.1:4000
ORIGA_CDN_BASE_URL=http://localhost:8080
```

Локально прогон идёт против dev-сборки (`trunk serve`). Сценарий
`failure_paths` («Недоступный CDN…») осмыслен только против release-сборки
(словари в dev бандлятся локально) — запускай его с `CI=true npx playwright
test --project=bdd`, чтобы webServer отдал `origa_ui/dist`, как в CI.

## Структура

```text
end2end/
├── bdd/features/        # .feature (русский Gherkin) — источник сценариев
├── bdd/steps/           # определения шагов (реюзают Page Objects)
├── bdd/fixtures.ts      # testUser/page-фикстуры + обменники шагов
├── pages/               # Page Objects (селекторы только по data-testid)
├── helpers/             # auth, lesson, navigation, cleanup, paths, http
├── fixtures/            # admin-токены, test data (sample.apkg, .wav)
├── scripts/             # check-ci-matrix.mjs — guard CI-матрицы
├── config.ts            # читает .env
├── global-setup.ts      # запуск TrailBase, admin-пароль, клинап сирот
└── global-teardown.ts   # остановка TrailBase
```

## CI-матрица и guard

CI гоняет BDD пятью группами (`ci.yml`, job `e2e`), каждая — по якорному
паттерну имени сгенерированного спек-файла (`features/<name>.feature.spec.js`
в полном титуле теста). Инвариант «каждый сценарий бегает ровно один раз»
проверяет `scripts/check-ci-matrix.mjs` (CI-джоба `e2e-matrix-guard`,
локально — `npm run guard:matrix`). Добавил `.feature` — добавь фичу в
паттерн группы, иначе guard уронит сборку.

Гарантии guard: отсутствие мёртвых/продублированных по грепу сценариев.
Семантические дубли внутри группы — зона ревью.

## Пользователи

| Пользователь | Email | Пароль | Назначение |
|---|---|---|---|
| Admin | `admin@localhost` | `secret` | API-операции, управление юзерами |
| Ручной | `uwuwu@uwuwu.net` | `uwuwu` | ручной UI-тестинг (создан заранее) |
| E2E | `e2e-<ts>-<rand>@origa.local` | `e2e-test-password-123` | создаётся/удаётся на каждый тест |

См. `end2end/AGENTS.md` (конвенции) и корневой `AGENTS.md` (общие правила).
