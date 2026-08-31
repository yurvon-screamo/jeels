import path from "node:path";
import { fileURLToPath } from "node:url";

/**
 * Absolute path of the `end2end/` directory.
 *
 * ESM replacement for `__dirname`: with `"type": "module"` Node runs the
 * Playwright-loaded TS sources as true ES modules where `__dirname` is
 * undefined — resolve fixture/pid paths from `import.meta.url` instead.
 */
export const end2endRoot = path.resolve(
    path.dirname(fileURLToPath(import.meta.url)),
    "..",
);
