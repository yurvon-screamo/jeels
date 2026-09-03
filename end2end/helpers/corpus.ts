import { readFileSync } from "node:fs";
import { join } from "node:path";

import { end2endRoot } from "./paths";

/**
 * Reads a stable vocabulary word from the N1 well-known set shipped on the
 * CDN (the same corpus the app imports during onboarding). Stress
 * assertions use corpus-derived data instead of hardcoded words, so
 * dictionary refreshes cannot break the test.
 *
 * The word is picked deterministically (the middle entry) to stay clear of
 * any first/last-position quirks in rendering.
 */
export function corpusWordFromCdn(): string {
    const path = join(end2endRoot, "..", "cdn", "well_known_set", "jlpt_n1.json");
    const data = JSON.parse(readFileSync(path, "utf8")) as { words: string[] };
    const words = data.words;
    if (!Array.isArray(words) || words.length === 0) {
        throw new Error(`N1 corpus at ${path} has no words`);
    }
    return words[Math.floor(words.length / 2)];
}
