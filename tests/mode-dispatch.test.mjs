import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

import { dispatchMode } from "../_lib/mode-dispatch.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "..");
const MODE_CONTRACT = readFileSync(
  path.join(ROOT, "canonical/mode-contract.md"),
  "utf8",
);
const MODE_DISPATCH = readFileSync(
  path.join(ROOT, "_lib/mode-dispatch.mjs"),
  "utf8",
);

const repoCtx = { hasPackageJson: true, hasTsconfig: true, hasSrcTree: true };

function fencedBlock(section) {
  const rx = new RegExp(
    `### ${section.replace(".", "\\.")}[\\s\\S]*?\`\`\`\\r?\\n([\\s\\S]*?)\`\`\``,
  );
  const match = MODE_CONTRACT.match(rx);
  return match?.[1] ?? "";
}

function contractTerms(section) {
  const terms = [];
  for (const rawLine of fencedBlock(section).trim().split(/\r?\n/)) {
    const line = rawLine.trim();
    if (!line) continue;
    if (line.includes('"') || line.includes(",")) {
      terms.push(
        ...[...line.matchAll(/"([^"]+)"|[^\s,]+/g)]
          .map((m) => m[1] ?? m[0])
          .filter(Boolean),
      );
    } else {
      terms.push(...line.split(/\s{2,}/).filter(Boolean));
    }
  }
  return terms;
}

function dispatchTerms(constName) {
  const rx = new RegExp(
    `const ${constName} = Object\\.freeze\\(\\[([\\s\\S]*?)\\]\\);`,
  );
  const body = MODE_DISPATCH.match(rx)?.[1] ?? "";
  return [...body.matchAll(/'([^']+)'/g)].map((m) => m[1]);
}

describe("mode dispatch canonical vocabulary", () => {
  it("M1. Korean verb list mirrors canonical §3.1", () => {
    expect(dispatchTerms("KOREAN_VERBS")).toEqual(contractTerms("3.1 Korean"));
  });

  it("M2. English verb list mirrors canonical §3.2", () => {
    expect(dispatchTerms("ENGLISH_VERBS")).toEqual(
      contractTerms("3.2 English"),
    );
  });

  it("M3. Korean guard list mirrors canonical §3.4", () => {
    expect(dispatchTerms("KOREAN_GUARDS")).toEqual(
      contractTerms("3.4 Guards").filter((x) => /[가-힣]/.test(x)),
    );
  });

  it("M4. English guard list mirrors canonical §3.4", () => {
    expect(dispatchTerms("ENGLISH_GUARDS")).toEqual(
      contractTerms("3.4 Guards").filter((x) => !/[가-힣]/.test(x)),
    );
  });
});

describe("mode dispatch guard and trigger cases", () => {
  it('T1. "찾아줘" alone → mode:none', () => {
    const r = dispatchMode("찾아줘", repoCtx);
    expect(r.mode).toBe("none");
  });

  it("T1b. nonTriggerReason is guard-only", () => {
    const r = dispatchMode("찾아줘", repoCtx);
    expect(r.nonTriggerReason).toBe("guard-only");
  });

  it("T1c. matched guards include 찾아줘", () => {
    const r = dispatchMode("찾아줘", repoCtx);
    expect(r.matchedGuards).toContain("찾아줘");
  });

  it('T2. "explain this" → mode:none', () => {
    const r = dispatchMode("explain this", repoCtx);
    expect(r.mode).toBe("none");
  });

  it("T2b. nonTriggerReason is guard-only OR pure-inspection", () => {
    const r = dispatchMode("explain this", repoCtx);
    expect(["guard-only", "pure-inspection"]).toContain(r.nonTriggerReason);
  });

  it('T3. "보여줘" alone → mode:none', () => {
    const r = dispatchMode("보여줘", repoCtx);
    expect(r.mode).toBe("none");
  });

  it('T4. "만들어줘" alone → mode:pre-write', () => {
    const r = dispatchMode("만들어줘", repoCtx);
    expect(r.mode).toBe("pre-write");
  });

  it("T4b. matched verbs include 만들어줘", () => {
    const r = dispatchMode("만들어줘", repoCtx);
    expect(r.matchedVerbs).toContain("만들어줘");
  });

  it("T4c. compoundGuardPlusVerb is false (no guard)", () => {
    const r = dispatchMode("만들어줘", repoCtx);
    expect(r.compoundGuardPlusVerb).toBe(false);
  });

  it('T5. "implement ..." → mode:pre-write', () => {
    const r = dispatchMode("implement a new function", repoCtx);
    expect(r.mode).toBe("pre-write");
  });

  it('T6. "찾아서 + 연결해줘" → mode:pre-write (verb wins)', () => {
    const r = dispatchMode("기존 helper 찾아서 연결해줘", repoCtx);
    expect(r.mode).toBe("pre-write");
  });

  it("T6b. compoundGuardPlusVerb is true", () => {
    const r = dispatchMode("기존 helper 찾아서 연결해줘", repoCtx);
    expect(r.compoundGuardPlusVerb).toBe(true);
  });

  it("T6c. both guard and verb matched", () => {
    const r = dispatchMode("기존 helper 찾아서 연결해줘", repoCtx);
    expect(r.matchedGuards.length).toBeGreaterThan(0);
    expect(r.matchedVerbs.length).toBeGreaterThan(0);
  });

  it('T7. English "find and refactor" → mode:pre-write, compound', () => {
    const r = dispatchMode("find and refactor this module", repoCtx);
    expect(r.mode).toBe("pre-write");
    expect(r.compoundGuardPlusVerb).toBe(true);
  });
});

describe("mode dispatch non-trigger precedence", () => {
  it("T8. verb + no repo context → mode:none", () => {
    const r = dispatchMode("만들어줘", {
      hasPackageJson: false,
      hasTsconfig: false,
      hasSrcTree: false,
    });
    expect(r.mode).toBe("none");
  });

  it("T8b. nonTriggerReason is no-repo-context", () => {
    const r = dispatchMode("만들어줘", {
      hasPackageJson: false,
      hasTsconfig: false,
      hasSrcTree: false,
    });
    expect(r.nonTriggerReason).toBe("no-repo-context");
  });

  it("T9. verb + only package.json → mode:pre-write (context satisfied)", () => {
    const r = dispatchMode("만들어줘", {
      hasPackageJson: true,
      hasTsconfig: false,
      hasSrcTree: false,
    });
    expect(r.mode).toBe("pre-write");
  });

  it('T10. "README 다듬어줘" → mode:none (prose-rewrite canonical rule)', () => {
    const r = dispatchMode("README 다듬어줘", repoCtx);
    expect(r.mode).toBe("none");
  });

  it("T10b. nonTriggerReason is prose-rewrite", () => {
    const r = dispatchMode("README 다듬어줘", repoCtx);
    expect(r.nonTriggerReason).toBe("prose-rewrite");
  });

  it('T11. "CHANGELOG 업데이트" → mode:none (prose)', () => {
    const r = dispatchMode("CHANGELOG 업데이트해줘", repoCtx);
    expect(r.mode).toBe("none");
    expect(r.nonTriggerReason).toBe("prose-rewrite");
  });

  it('T12. "docs/*.md 다듬어줘" → mode:none (prose path)', () => {
    const r = dispatchMode("docs/architecture.md 다듬어줘", repoCtx);
    expect(r.mode).toBe("none");
    expect(r.nonTriggerReason).toBe("prose-rewrite");
  });

  it('T13. "rewrite the README" → mode:none', () => {
    const r = dispatchMode("rewrite the README", repoCtx);
    expect(r.mode).toBe("none");
    expect(r.nonTriggerReason).toBe("prose-rewrite");
  });

  it('T14. "주석 오타 고쳐줘" → mode:none (comment-typo-fix)', () => {
    const r = dispatchMode("주석 오타 고쳐줘", repoCtx);
    expect(r.mode).toBe("none");
    expect(r.nonTriggerReason).toBe("comment-typo-fix");
  });

  it('T15. "fix comment typo" → mode:none', () => {
    const r = dispatchMode("fix comment typo", repoCtx);
    expect(r.mode).toBe("none");
    expect(r.nonTriggerReason).toBe("comment-typo-fix");
  });

  it('T16. "fix this bug" → mode:pre-write (generic fix, not comment-typo)', () => {
    const r = dispatchMode("fix this bug", repoCtx);
    expect(r.mode).toBe("pre-write");
  });

  it('T17. "이 코드 어떻게 동작해요?" → mode:none (pure-inspection)', () => {
    const r = dispatchMode("이 코드 어떻게 동작해요?", repoCtx);
    expect(r.mode).toBe("none");
  });

  it("T18. no-repo-context beats prose-rewrite in precedence", () => {
    const r = dispatchMode("README 다듬어줘", {
      hasPackageJson: false,
      hasTsconfig: false,
      hasSrcTree: false,
    });
    expect(r.mode).toBe("none");
    expect(r.nonTriggerReason).toBe("no-repo-context");
  });
});

describe("mode dispatch return shape and purity", () => {
  it("T19. trigger result has string rationale", () => {
    const r = dispatchMode("만들어줘", repoCtx);
    expect(r.rationale).toEqual(expect.any(String));
    expect(r.rationale.length).toBeGreaterThan(0);
  });

  it("T19b. trigger result has matchedVerbs array", () => {
    const r = dispatchMode("만들어줘", repoCtx);
    expect(Array.isArray(r.matchedVerbs)).toBe(true);
  });

  it("T19c. trigger result has matchedGuards array (possibly empty)", () => {
    const r = dispatchMode("만들어줘", repoCtx);
    expect(Array.isArray(r.matchedGuards)).toBe(true);
  });

  it("T19d. trigger result has compoundGuardPlusVerb boolean", () => {
    const r = dispatchMode("만들어줘", repoCtx);
    expect(typeof r.compoundGuardPlusVerb).toBe("boolean");
  });

  it("T19e. trigger result does NOT carry nonTriggerReason", () => {
    const r = dispatchMode("만들어줘", repoCtx);
    expect(r.nonTriggerReason).toBeUndefined();
  });

  it("T20. pure function: deterministic mode", () => {
    const a = dispatchMode("만들어줘", repoCtx);
    const b = dispatchMode("만들어줘", repoCtx);
    expect(a.mode).toBe(b.mode);
  });

  it("T20b. pure function: deterministic rationale", () => {
    const a = dispatchMode("만들어줘", repoCtx);
    const b = dispatchMode("만들어줘", repoCtx);
    expect(a.rationale).toBe(b.rationale);
  });
});
