import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "..");
const workflow = readFileSync(
  path.join(ROOT, ".github", "workflows", "ci.yml"),
  "utf8",
);

function sectionAfter(marker) {
  const idx = workflow.indexOf(marker);
  return idx === -1 ? "" : workflow.slice(idx);
}

describe("GitHub Actions CI policy", () => {
  it("keeps manual CI dispatch available", () => {
    expect(workflow).toMatch(/^\s*workflow_dispatch:\s*$/m);
  });

  it("runs pull request CI when drafts become ready for review", () => {
    const pullRequestSection = sectionAfter("pull_request:");

    expect(workflow).toMatch(
      /pull_request:\s*\n\s+types:\s*\[[^\]]*ready_for_review[^\]]*\]/m,
    );
    expect(pullRequestSection.slice(0, 240)).toContain("ready_for_review");
  });

  it("keeps normal pull request update events wired", () => {
    for (const event of ["opened", "synchronize", "reopened"]) {
      expect(workflow).toMatch(
        new RegExp(
          `pull_request:\\s*\\n\\s+types:\\s*\\[[^\\]]*${event}[^\\]]*\\]`,
          "m",
        ),
      );
    }
  });

  it("skips draft pull request runner work at the job gate", () => {
    const testJobSection = sectionAfter("  test:");

    expect(workflow).toMatch(
      /if:\s*\$\{\{\s*github\.event_name\s*!=\s*'pull_request'\s*\|\|\s*github\.event\.pull_request\.draft\s*==\s*false\s*\}\}/m,
    );
    expect(testJobSection.slice(0, 320)).toContain("pull_request");
  });

  it("detects changed surfaces before starting expensive CI jobs", () => {
    const changesJobSection = sectionAfter("  changes:");

    expect(changesJobSection.slice(0, 1200)).toContain(
      "Detect changed surfaces",
    );
    expect(changesJobSection.slice(0, 1200)).toContain("fetch-depth: 0");
    expect(changesJobSection.slice(0, 3000)).toContain(
      'echo "node=true" >> "$GITHUB_OUTPUT"',
    );
    expect(changesJobSection.slice(0, 3000)).toContain(
      'echo "rust=true" >> "$GITHUB_OUTPUT"',
    );
  });

  it("runs Node CI only when the path filter marks Node changes", () => {
    const testJobSection = sectionAfter("  test:");

    expect(testJobSection.slice(0, 420)).toContain("needs: changes");
    expect(testJobSection.slice(0, 420)).toContain(
      "needs.changes.outputs.node == 'true'",
    );
  });

  it("keeps push CI for main and master", () => {
    expect(workflow).toMatch(/push:\s*\n\s+branches:\s*\[main,\s*master\]/m);
  });

  it("runs Rust CI only when Rust-owned paths change", () => {
    const rustJobSection = sectionAfter("\n  rust:");
    const changesJobSection = sectionAfter("  changes:");

    expect(rustJobSection.slice(0, 420)).toContain("needs: changes");
    expect(rustJobSection.slice(0, 420)).toContain(
      "needs.changes.outputs.rust == 'true'",
    );
    expect(changesJobSection).toContain("experiments/rust-main/*");
    expect(changesJobSection).toContain("experiments/rust-sidecar/*");
    expect(changesJobSection).toContain(
      "tests/fixtures/m7-cargo-json-diagnostic-capture-v4/*",
    );
  });

  it("runs Rust cargo checks in CI", () => {
    expect(workflow).toContain(
      "cargo test --locked --manifest-path experiments/rust-main/rust-cargo-oracle/Cargo.toml",
    );
    expect(workflow).toContain(
      "cargo test --locked --manifest-path experiments/rust-sidecar/rust-source-health/Cargo.toml",
    );
  });
});
