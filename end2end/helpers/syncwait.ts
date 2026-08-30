import type { Page } from "@playwright/test";

/**
 * TrailBase record-writes for the user row — the persistence contract of
 * the sync layer. Single source of truth for steps that must wait until a
 * user-state mutation actually landed on the server (favourite toggles,
 * acquaintance hand close, …) before doing a full page reload.
 */
const RECORD_WRITE_METHODS = new Set(["POST", "PUT", "PATCH"]);
const USER_RECORD_URL = /\/api\/records\/v1\/(user|domain_user)(\/|$)/;

/**
 * Registers (BEFORE the triggering action — a fast save can slip past the
 * window otherwise) a wait for the next user-record write and returns the
 * promise. The caller performs the action, then awaits and asserts `ok()`.
 */
export function waitForUserRecordWrite(
    page: Page,
    timeout = 15_000,
): Promise<{ ok: () => boolean; status: () => number }> {
    return page.waitForResponse(
        (resp) => USER_RECORD_URL.test(resp.url()) && RECORD_WRITE_METHODS.has(resp.request().method()),
        { timeout },
    );
}
