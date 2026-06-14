import { spawnSync } from "node:child_process";
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { afterAll, beforeAll, describe, expect, it } from "vitest";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "..");
const NODE = process.execPath;
const SCRIPT = path.join(ROOT, "scripts/publish-public-plugin.mjs");
const PACKAGE = JSON.parse(
  readFileSync(path.join(ROOT, "package.json"), "utf8"),
);
const CURRENT_VERSION = PACKAGE.version;
const COMMAND_NAMES = [
  "audit",
  "canon-draft",
  "check-canon",
  "full",
  "lumin-repo-lens-lab",
  "post-write",
  "pre-write",
  "refactor-plan",
  "welcome",
];

function run(cmd, args, cwd = ROOT, options = {}) {
  return spawnSync(cmd, args, {
    cwd,
    env: { ...process.env, ...(options.env ?? {}) },
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  });
}

function git(args, cwd) {
  const result = run("git", args, cwd);
  if (result.status !== 0) {
    throw new Error(
      `git ${args.join(" ")} failed\n${result.stdout}\n${result.stderr}`,
    );
  }
  return result.stdout.trim();
}

function writeJson(file, value) {
  mkdirSync(path.dirname(file), { recursive: true });
  writeFileSync(file, `${JSON.stringify(value, null, 2)}\n`);
}

function writeText(file, text) {
  mkdirSync(path.dirname(file), { recursive: true });
  writeFileSync(file, text);
}

function createDistPlugin(root, version = CURRENT_VERSION) {
  writeJson(path.join(root, ".claude-plugin/plugin.json"), {
    name: "lumin-repo-lens-lab",
    version,
    description:
      "Lumin Repo Lens: evidence-backed TS/JS repo structure lens for Claude Code.",
    repository: "https://github.com/annyeong844/lumin-repo-lens-lab",
    license: "MIT",
  });
  writeJson(path.join(root, ".claude-plugin/marketplace.json"), {
    name: "annyeong844-marketplace",
    owner: { name: "annyeong844" },
    metadata: {
      description:
        "Public beta marketplace for annyeong844 Claude Code plugins.",
      version,
    },
    plugins: [
      {
        name: "lumin-repo-lens-lab",
        source: "./",
        description:
          "Lumin Repo Lens: evidence-backed TS/JS repo structure lens for Claude Code.",
      },
    ],
  });

  for (const commandName of COMMAND_NAMES) {
    writeText(
      path.join(root, "commands", `${commandName}.md`),
      `---\ndescription: ${commandName}\n---\n\nUse \${CLAUDE_PLUGIN_ROOT}/skills/lumin-repo-lens-lab/SKILL.md\n`,
    );
  }

  for (const skill of [
    "lumin-repo-lens-lab",
    "lumin-repo-lens-lab-write-gate",
    "lumin-repo-lens-lab-canon",
  ]) {
    writeText(
      path.join(root, "skills", skill, "SKILL.md"),
      `---\nname: ${skill}\n---\n`,
    );
  }
  writeJson(path.join(root, "hooks/hooks.json"), {
    hooks: {
      PreToolUse: [
        {
          matcher: "*",
          hooks: [
            {
              type: "command",
              command: 'node "${CLAUDE_PLUGIN_ROOT}/hooks/pre-tool-use.mjs"',
              timeout: 2,
            },
          ],
        },
      ],
    },
  });
  writeText(
    path.join(root, "hooks/_runner-utils.mjs"),
    "export const runnerUtils = true;\n",
  );
  writeText(path.join(root, "hooks/pre-tool-use.mjs"), "#!/usr/bin/env node\n");
  writeText(
    path.join(root, "hooks/post-tool-batch.mjs"),
    "#!/usr/bin/env node\n",
  );
  writeText(path.join(root, "hooks/stop.mjs"), "#!/usr/bin/env node\n");
  writeText(
    path.join(root, "hooks/user-prompt-submit.mjs"),
    "#!/usr/bin/env node\n",
  );
  writeJson(path.join(root, "skills/lumin-repo-lens-lab/package.json"), {
    name: "lumin-repo-lens-lab-skill",
    version,
    luminRepoLens: { distribution: "skill" },
  });
  writeJson(path.join(root, "skills/lumin-repo-lens-lab/package-lock.json"), {
    name: "lumin-repo-lens-lab-skill",
    version,
    lockfileVersion: 3,
    packages: { "": { name: "lumin-repo-lens-lab-skill", version } },
  });
  writeText(
    path.join(root, "skills/lumin-repo-lens-lab/_engine/producers/emit-sarif.mjs"),
    `const TOOL_VERSION = '${version}';\n`,
  );
  writeText(path.join(root, "README.plugin-package.md"), "# Package root\n");
}

function seedPublicRepo(workDir, bareDir) {
  mkdirSync(workDir, { recursive: true });
  git(["init", "-b", "main"], workDir);
  git(["config", "user.name", "annyeong844"], workDir);
  git(
    ["config", "user.email", "annyeong844@users.noreply.github.com"],
    workDir,
  );

  writeJson(path.join(workDir, ".claude-plugin/plugin.json"), {
    name: "lumin-repo-lens-lab",
    version: "0.9.0-beta.6",
  });
  writeJson(path.join(workDir, ".claude-plugin/marketplace.json"), {
    name: "annyeong844-marketplace",
    metadata: { version: "0.9.0-beta.6" },
  });
  writeJson(path.join(workDir, "skills/lumin-repo-lens-lab/package.json"), {
    name: "lumin-repo-lens-lab-skill",
    version: "0.9.0-beta.6",
  });
  writeText(
    path.join(workDir, "CHANGELOG.md"),
    [
      "# Changelog",
      "",
      "## 0.9.0-beta.6",
      "",
      "- Existing public beta6 entry.",
      "",
    ].join("\n"),
  );
  writeText(path.join(workDir, "README.md"), "# Old public README\n");
  writeText(path.join(workDir, "README.ko.md"), "# 오래된 공개 README\n");
  writeText(path.join(workDir, "LICENSE"), "MIT\n");
  writeText(path.join(workDir, ".gitignore"), "node_modules/\n.audit/\n");
  git(["add", "-A"], workDir);
  git(["commit", "-m", "seed public package"], workDir);
  git(["clone", "--bare", workDir, bareDir], path.dirname(bareDir));
}

describe("public plugin publisher", () => {
  let tmp;
  let dist;
  let seed;
  let bare;
  let dryCheckout;
  let pushCheckout;
  let dry;
  let pushed;

  beforeAll(() => {
    tmp = mkdtempSync(path.join(os.tmpdir(), "publish-public-plugin-vitest-"));
    dist = path.join(tmp, "dist-plugin");
    seed = path.join(tmp, "seed-public");
    bare = path.join(tmp, "public.git");
    dryCheckout = path.join(tmp, "dry-checkout");
    pushCheckout = path.join(tmp, "push-checkout");
    createDistPlugin(dist);
    seedPublicRepo(seed, bare);

    dry = run(NODE, [
      SCRIPT,
      "--repo",
      bare,
      "--dist",
      dist,
      "--checkout-dir",
      dryCheckout,
      "--no-build",
      "--dry-run",
      "--keep-checkout",
    ]);

    pushed = run(
      NODE,
      [
        SCRIPT,
        "--repo",
        bare,
        "--dist",
        dist,
        "--checkout-dir",
        pushCheckout,
        "--no-build",
        "--push",
      ],
      ROOT,
      {
        env: {
          LUMIN_REPO_LENS_PUBLISH_AUTHOR_NAME: "Lumin Publish Bot",
          LUMIN_REPO_LENS_PUBLISH_AUTHOR_EMAIL: "lumin-publish@example.test",
        },
      },
    );
  }, 30000);

  afterAll(() => {
    if (tmp) rmSync(tmp, { recursive: true, force: true });
  });

  it("dry-run exits successfully", () => {
    expect(`${dry.stdout}\n${dry.stderr}`).toBeDefined();
    expect(dry.status).toBe(0);
  });

  it("dry-run stages plugin package version without committing", () => {
    expect(
      JSON.parse(
        readFileSync(
          path.join(dryCheckout, ".claude-plugin/plugin.json"),
          "utf8",
        ),
      ).version,
    ).toBe(CURRENT_VERSION);
    expect(
      JSON.parse(
        readFileSync(
          path.join(dryCheckout, "skills/lumin-repo-lens-lab/package.json"),
          "utf8",
        ),
      ).version,
    ).toBe(CURRENT_VERSION);
    expect(git(["rev-parse", "HEAD"], dryCheckout)).toBe(
      git(["rev-parse", "main"], seed),
    );
  });

  it("dry-run does not leak maintainer-only root directories", () => {
    for (const rel of [
      "docs",
      "tests",
      "_lib",
      "skills/lumin-repo-lens-lab-codex",
    ]) {
      expect(existsSync(path.join(dryCheckout, rel))).toBe(false);
    }
  });

  it("dry-run prepends internal beta entries before the existing public beta6 entry", () => {
    const dryChangelog = readFileSync(
      path.join(dryCheckout, "CHANGELOG.md"),
      "utf8",
    );

    expect(dryChangelog.indexOf(`## ${CURRENT_VERSION}`)).toBeLessThan(
      dryChangelog.indexOf("## 0.9.0-beta.6"),
    );
    expect(dryChangelog).toContain("## 0.9.0-beta.10");
    expect(dryChangelog).toContain("- Existing public beta6 entry.");
  });

  it("dry-run syncs the public package CI workflow", () => {
    const workflowPath = path.join(dryCheckout, ".github/workflows/ci.yml");
    const workflowText = existsSync(workflowPath)
      ? readFileSync(workflowPath, "utf8")
      : "";

    expect(existsSync(workflowPath)).toBe(true);
    expect(workflowText).toContain("npm ci");
    expect(workflowText).toContain("npm run smoke");
    expect(workflowText).toContain(
      "node skills/lumin-repo-lens-lab/scripts/audit-repo.mjs --help",
    );
    expect(workflowText).toContain("node hooks/pre-tool-use.mjs");
  });

  it("keeps maintainer-only paths out of the public package CI workflow", () => {
    const workflowText = readFileSync(
      path.join(dryCheckout, ".github/workflows/ci.yml"),
      "utf8",
    );

    expect(workflowText).not.toMatch(
      /\b(tests|test-harness|docs\/spec|_lib|p6-corpus)\b/,
    );
  });

  it("dry-run syncs auto-hook manifest and runner scripts", () => {
    for (const rel of [
      "hooks/hooks.json",
      "hooks/_runner-utils.mjs",
      "hooks/pre-tool-use.mjs",
      "hooks/post-tool-batch.mjs",
      "hooks/stop.mjs",
      "hooks/user-prompt-submit.mjs",
    ]) {
      expect(existsSync(path.join(dryCheckout, rel))).toBe(true);
    }
  });

  it("push mode commits and pushes to public main", () => {
    expect(`${pushed.stdout}\n${pushed.stderr}`).toBeDefined();
    expect(pushed.status).toBe(0);
    expect(pushed.stdout).toContain("pushed public package");
  });

  it("push mode honors explicit publish author environment", () => {
    expect(
      git(["--git-dir", bare, "log", "-1", "--format=%an <%ae>"], ROOT),
    ).toBe("Lumin Publish Bot <lumin-publish@example.test>");
  });

  it("pushed public repo exposes current plugin and skill metadata", () => {
    const pushedPlugin = git(
      ["--git-dir", bare, "show", "main:.claude-plugin/plugin.json"],
      ROOT,
    );
    const pushedSkillPkg = git(
      ["--git-dir", bare, "show", "main:skills/lumin-repo-lens-lab/package.json"],
      ROOT,
    );

    expect(JSON.parse(pushedPlugin).version).toBe(CURRENT_VERSION);
    expect(JSON.parse(pushedSkillPkg).version).toBe(CURRENT_VERSION);
  });

  it("pushed public repo includes package CI workflow", () => {
    const result = run(
      "git",
      ["--git-dir", bare, "show", "main:.github/workflows/ci.yml"],
      ROOT,
    );

    expect(`${result.stdout}\n${result.stderr}`).toBeDefined();
    expect(result.status).toBe(0);
    expect(result.stdout).toContain("name: Public Package CI");
    expect(result.stdout).toContain(
      "working-directory: skills/lumin-repo-lens-lab",
    );
    expect(result.stdout).toContain("npm run smoke");
    expect(result.stdout).toContain("node hooks/pre-tool-use.mjs");
  });

  it("pushed public repo includes auto-hook manifest", () => {
    const result = run(
      "git",
      ["--git-dir", bare, "show", "main:hooks/hooks.json"],
      ROOT,
    );

    expect(`${result.stdout}\n${result.stderr}`).toBeDefined();
    expect(result.status).toBe(0);
    expect(JSON.parse(result.stdout).hooks?.PreToolUse).toBeTruthy();
  });

  it("package.json exposes check and push scripts for public plugin publishing", () => {
    expect(PACKAGE.scripts["check:public-plugin"]).toBe(
      "node scripts/publish-public-plugin.mjs --dry-run",
    );
    expect(PACKAGE.scripts["publish:public-plugin"]).toBe(
      "node scripts/publish-public-plugin.mjs --push",
    );
  });
});
