#!/usr/bin/env node
/**
 * CI e2e-matrix guard: every BDD scenario must run in the CI matrix
 * exactly once.
 *
 * How it works:
 * 1. Regenerates `.features-gen/` via bddgen (guards against stale
 *    generated specs drifting from `.feature` files).
 * 2. Parses the scenario inventory from `bdd/features/*.feature`.
 * 3. Parses the grep patterns of the `e2e` job matrix from
 *    `.github/workflows/ci.yml` (structural validation: matrix groups
 *    must match case labels, every case arm must carry a `--grep`).
 * 4. For every group runs `playwright test --project=bdd --list
 *    --reporter=json --grep <pattern>` (no browser, no webServer,
 *    no globalSetup — listing only).
 * 5. Asserts the invariant: each scenario is matched by EXACTLY one
 *    group. Reports dead (unmatched) and duplicated scenarios.
 *
 * Guarantee boundaries (IMPORTANT):
 * - This guard catches grep-level holes only: scenarios that never run
 *   in CI or run in several groups. It was born from a real incident:
 *   the `Фразы$` anchor silently excluded the whole phrases.feature.
 * - Semantic duplicates (two scenarios inside the SAME group testing
 *   the same behaviour) are OUT OF SCOPE — they are caught by review.
 *
 * Expected output on success: per-group counts, total, exit 0.
 */

import { spawnSync } from "node:child_process";
import { readdir, readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const END2END_ROOT = path.dirname(path.dirname(fileURLToPath(import.meta.url)));
const REPO_ROOT = path.dirname(END2END_ROOT);
const CI_YML_PATH = path.join(REPO_ROOT, ".github", "workflows", "ci.yml");
const FEATURES_DIR = path.join(END2END_ROOT, "bdd", "features");
const PW_CLI = path.join("node_modules", "@playwright", "test", "cli.js");

const violations = [];
const fail = (message) => violations.push(message);

// Known @fixme scenarios (review follow-up: a fixme without a reminder
// mechanism silently rots). Adding a new @fixme requires an explicit entry
// here; removing a fixme requires removing its entry — the guard fails
// loudly either way. Keep entries coupled to their tracking issues.
const KNOWN_FIXME = new Set<string>([]);

// ---------------------------------------------------------------------------
// 1. Fresh bddgen output (guards against stale .features-gen)
// ---------------------------------------------------------------------------
const bddPkg = JSON.parse(
    await readFile(
        path.join(END2END_ROOT, "node_modules", "playwright-bdd", "package.json"),
        "utf8",
    ),
);
const bddGenEntry = path.join(
    END2END_ROOT,
    "node_modules",
    "playwright-bdd",
    bddPkg.bin.bddgen.replace(/^\.\//, ""),
);
const gen = spawnSync("node", [bddGenEntry], {
    cwd: END2END_ROOT,
    encoding: "utf8",
});
if (gen.status !== 0) {
    console.error(gen.stdout);
    console.error(gen.stderr);
    console.error("FAIL: bddgen exited non-zero — cannot verify a stale matrix.");
    process.exit(1);
}

// ---------------------------------------------------------------------------
// 2. Scenario inventory from .feature files
// ---------------------------------------------------------------------------
const featureFiles = (await readdir(FEATURES_DIR))
    .filter((name) => name.endsWith(".feature"))
    .sort();

const expected = new Set();
const actualFixme = new Set();
for (const name of featureFiles) {
    const text = await readFile(path.join(FEATURES_DIR, name), "utf8");
    // Tag lines carry over to the following scenario; @fixme marks it.
    let pendingTags = "";
    for (const line of text.split("\n")) {
        const tags = line.match(/^\s*(@\S+(?:\s+@\S+)*)\s*$/);
        if (tags) {
            pendingTags = tags[1];
            continue;
        }
        // Russian Gherkin: «Сценарий:». Scenario Outlines («Структура
        // сценария:») are NOT supported by this guard: playwright-bdd
        // titles outline examples differently, which would surface as
        // false unknown/DEAD. Fail loudly instead of lying silently.
        if (/^\s*Структура сценария:/.test(line)) {
            fail(
                `${name} uses a Scenario Outline — extend the guard inventory ` +
                    `before adding outlines (example titling differs)`,
            );
            continue;
        }
        const match = line.match(/^\s*Сценарий:\s*(.+?)\s*$/);
        if (match) {
            expected.add(`${name}|${match[1]}`);
            if (pendingTags.includes("@fixme")) actualFixme.add(`${name}|${match[1]}`);
            pendingTags = "";
        }
    }
}
if (expected.size === 0) fail(`no scenarios found in ${FEATURES_DIR}`);

// ---------------------------------------------------------------------------
// 3. Matrix groups and grep patterns from ci.yml
// ---------------------------------------------------------------------------
const ciYml = await readFile(CI_YML_PATH, "utf8");

const matrixBlock = ciYml.match(/matrix:\s*\n\s*group:\s*\n((?:\s*-\s+[\w-]+\s*\n)+)/);
const matrixGroups = matrixBlock
    ? [...matrixBlock[1].matchAll(/-\s+([\w-]+)/g)].map((m) => m[1])
    : [];
if (matrixGroups.length === 0)
    fail("could not parse `matrix.group` entries from ci.yml (unexpected format)");

const caseBlock = ciYml.match(/case "\$\{\{ matrix\.group \}\}" in([\s\S]*?)\n\s*esac/);
const caseLabels = caseBlock
    ? [...caseBlock[1].matchAll(/^\s*([\w-]+)\)\s*$/gm)].map((m) => m[1])
    : [];
if (caseLabels.length === 0)
    fail('could not parse `case "${{ matrix.group }}"` arms from ci.yml (unexpected format)');

// Per-arm extraction (not two parallel arrays): a grep is attributed to
// the arm block it lives in, so two greps in one arm and none in another
// produce an explicit "arm without grep" violation instead of silently
// shifting the group mapping.
const armBlocks = caseBlock
    ? caseBlock[1].split(/^\s*([\w-]+)\)\s*$/m).slice(1)
    : [];
const groupPatterns = new Map();
for (let i = 0; i + 1 < armBlocks.length; i += 2) {
    const label = armBlocks[i];
    const greps = [...armBlocks[i + 1].matchAll(/--grep "([^"]+)"/g)].map((m) => m[1]);
    if (greps.length !== 1) {
        fail(`case arm "${label}" must carry exactly one --grep, found ${greps.length}`);
        continue;
    }
    groupPatterns.set(label, greps[0]);
}

// Structural validation: every arm must bind to a declared group and own a grep.
const labelSet = new Set(caseLabels);
for (const group of matrixGroups)
    if (!labelSet.has(group)) fail(`matrix group "${group}" has no case arm in ci.yml`);
for (const label of caseLabels)
    if (!matrixGroups.includes(label))
        fail(`case arm "${label}" is not a declared matrix group in ci.yml`);

for (const [label, pattern] of groupPatterns) {
    try {
        new RegExp(pattern);
    } catch (error) {
        fail(`group "${label}" has an invalid --grep regex: ${error.message}`);
    }
}

// ---------------------------------------------------------------------------
// 4. Listing per group
// ---------------------------------------------------------------------------
/** @param {string} pattern @returns {Set<string>} `${featureFile}|${scenario}` keys */
function listScenarioKeys(pattern) {
    const result = spawnSync(
        "node",
        [PW_CLI, "test", "--project=bdd", "--list", "--reporter=json", `--grep=${pattern}`],
        { cwd: END2END_ROOT, encoding: "utf8", maxBuffer: 128 * 1024 * 1024 },
    );
    if (result.status !== 0) {
        console.error(result.stdout);
        console.error(result.stderr);
        fail(`playwright --list failed for pattern: ${pattern}`);
        return new Set();
    }
    // The JSON report is pretty-printed starting with "{" on its own line;
    // Playwright's dotenv tip ("{ debug: true }") contains braces inline,
    // so anchor on a line consisting of exactly "{".
    const jsonMatch = result.stdout.match(/^\{$/m);
    if (!jsonMatch || jsonMatch.index === undefined) {
        fail(`playwright --list produced no JSON for pattern: ${pattern}`);
        return new Set();
    }
    let data;
    try {
        data = JSON.parse(result.stdout.slice(jsonMatch.index));
    } catch (error) {
        fail(
            `playwright --list produced unparseable JSON for pattern "${pattern}": ` +
                error.message,
        );
        return new Set();
    }
    const keys = new Set();
    for (const top of data.suites ?? []) {
        const featureFile = path
            .basename(String(top.title ?? top.file ?? ""))
            .replace(/\.spec\.js$/, "");
        for (const suite of [top, ...(top.suites ?? [])]) {
            for (const spec of suite.specs ?? []) keys.add(`${featureFile}|${spec.title}`);
        }
    }
    return keys;
}

const matchedBy = new Map(); // key -> [groups]
const groupSizes = [];
for (const [group, pattern] of groupPatterns) {
    const keys = listScenarioKeys(pattern);
    groupSizes.push([group, keys.size]);
    for (const key of keys) {
        const owners = matchedBy.get(key) ?? [];
        owners.push(group);
        matchedBy.set(key, owners);
    }
}

// ---------------------------------------------------------------------------
// 5. Invariants
// ---------------------------------------------------------------------------
for (const [key, owners] of matchedBy) {
    if (!expected.has(key)) {
        fail(`group(s) ${owners.join(", ")} match an unknown scenario "${key}" — ` +
            `stale .features-gen or renamed scenario?`);
    } else if (owners.length > 1) {
        fail(`scenario "${key}" runs in ${owners.length} groups: ${owners.join(", ")}`);
    }
}
for (const key of expected) {
    if (!matchedBy.has(key)) fail(`DEAD scenario — no CI group runs "${key}"`);
}

// Known-fixme inventory: exact match with the tagged set.
for (const key of actualFixme)
    if (!KNOWN_FIXME.has(key))
        fail(`new @fixme "${key}" — track it in an issue and add to KNOWN_FIXME in this script`);
for (const key of KNOWN_FIXME)
    if (!actualFixme.has(key))
        fail(`KNOWN_FIXME entry "${key}" is no longer tagged @fixme — remove the entry`);

// Report
const width = Math.max(...groupSizes.map(([name]) => name.length), 5);
for (const [group, size] of groupSizes)
    console.log(`  ${group.padEnd(width)} : ${size} scenarios`);
console.log(`  total: ${[...matchedBy.keys()].length} / ${expected.size} expected`);

if (violations.length > 0) {
    console.error(`\nCI matrix guard FAILED (${violations.length} violation(s)):`);
    for (const v of violations) console.error(`  - ${v}`);
    process.exit(1);
}
console.log("\nCI matrix guard OK: every scenario runs exactly once.");
