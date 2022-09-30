import { tmpdir } from "node:os";
import { mkdtempSync } from "node:fs";
import { join } from "node:path";

export const tempDir = mkdtempSync(join(tmpdir(), "fig-test-"));
