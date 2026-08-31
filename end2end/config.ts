import { config } from "dotenv";
import { resolve } from "path";

import { end2endRoot } from "./helpers/paths";

// Override system env vars with local .env values
config({ path: resolve(end2endRoot, ".env"), override: true });

export function getTrailBaseUrl(): string {
    const url = process.env.TRAILBASE_URL;
    if (!url) {
        throw new Error("TRAILBASE_URL is not set. Configure it in .env file.");
    }
    return url;
}
