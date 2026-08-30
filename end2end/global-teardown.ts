import * as fs from "fs";
import { end2endRoot } from "./helpers/paths";
import * as path from "path";

export default async function globalTeardown(): Promise<void> {
    console.log("[global-teardown] Starting teardown...");

    // Kill TrailBase process
    const pidFile = path.join(end2endRoot, ".trailbase.pid");
    if (fs.existsSync(pidFile)) {
        const pid = parseInt(fs.readFileSync(pidFile, "utf-8").trim(), 10);
        if (!isNaN(pid)) {
            try {
                process.kill(pid);
                console.log(`[global-teardown] Killed TrailBase (PID ${pid})`);
            } catch {
                // Process may have already exited
            }
        }
        fs.unlinkSync(pidFile);
    }

    console.log("[global-teardown] Done.");
}
