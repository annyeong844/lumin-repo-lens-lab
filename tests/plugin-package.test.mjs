import { spawnSync } from "node:child_process";
import {
  cpSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  rmSync,
} from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { afterAll, beforeAll, describe, expect, it } from "vitest";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REAL_ROOT = path.resolve(__dirname, "..");
const NODE = process.execPath;
const COMMAND_SKILL_TARGETS = {
  "lumin-repo-lens-lab": "skills/lumin-repo-lens-lab/SKILL.md",
  welcome: "skills/lumin-repo-lens-lab/SKILL.md",
  full: "skills/lumin-repo-lens-lab/SKILL.md",
  audit: "skills/lumin-repo-lens-lab/SKILL.md",
  "refactor-plan": "skills/lumin-repo-lens-lab/SKILL.md",
  "pre-write": "skills/lumin-repo-lens-lab-write-gate/SKILL.md",
  "post-write": "skills/lumin-repo-lens-lab-write-gate/SKILL.md",
  "canon-draft": "skills/lumin-repo-lens-lab-canon/SKILL.md",
  "check-canon": "skills/lumin-repo-lens-lab-canon/SKILL.md",
};

function pluginRefsFromCommand(commandText) {
  return [...commandText.matchAll(/\$\{CLAUDE_PLUGIN_ROOT\}\/([^\s`,]+)/g)].map(
    (match) => match[1],
  );
}

function copyRepoFixture(srcRoot, destRoot) {
  cpSync(srcRoot, destRoot, {
    recursive: true,
    filter: (src) => {
      const rel = path.relative(srcRoot, src).replace(/\\/g, "/");
      if (!rel) return true;
      return !(
        rel === ".git" ||
        rel.startsWith(".git/") ||
        rel === "node_modules" ||
        rel.startsWith("node_modules/") ||
        rel === ".hex-skills" ||
        rel.startsWith(".hex-skills/") ||
        rel === "dist" ||
        rel.startsWith("dist/") ||
        rel === "output" ||
        rel.startsWith("output/") ||
        rel === "p6-corpus" ||
        rel.startsWith("p6-corpus/") ||
        rel.startsWith("review-output-") ||
        rel.startsWith(".audit/")
      );
    },
  });
}

describe("plugin package build output", () => {
  let tmp;
  let fixtureRoot;
  let buildPlugin;
  let out;
  let legacyOut;
  let build;

  beforeAll(() => {
    tmp = mkdtempSync(path.join(os.tmpdir(), "plugin-package-vitest-"));
    fixtureRoot = path.join(tmp, "repo");
    copyRepoFixture(REAL_ROOT, fixtureRoot);
    buildPlugin = path.join(fixtureRoot, "scripts/build-plugin-package.mjs");
    out = path.join(tmp, "lumin-repo-lens-lab-plugin");
    legacyOut = path.join(tmp, "lumin-audit-plugin");
    mkdirSync(legacyOut, { recursive: true });
    build = spawnSync(NODE, [buildPlugin, "--out", out], {
      cwd: fixtureRoot,
      encoding: "utf8",
    });
  });

  afterAll(() => {
    if (tmp) rmSync(tmp, { recursive: true, force: true });
  });

  it("build-plugin-package exits successfully", () => {
    expect(`${build.stdout}\n${build.stderr}`).toBeDefined();
    expect(build.status).toBe(0);
  });

  it("includes Claude Code plugin metadata and commands", () => {
    expect(existsSync(path.join(out, ".claude-plugin/plugin.json"))).toBe(true);
    expect(existsSync(path.join(out, ".claude-plugin/marketplace.json"))).toBe(
      true,
    );
    expect(existsSync(path.join(out, "commands/lumin-repo-lens-lab.md"))).toBe(
      true,
    );
    expect(existsSync(path.join(out, "commands/pre-write.md"))).toBe(true);
    expect(existsSync(path.join(out, "commands/check-canon.md"))).toBe(true);
  });

  it("includes the auto-hook manifest", () => {
    expect(existsSync(path.join(out, "hooks/hooks.json"))).toBe(true);
  });

  it("includes auto-hook runner scripts", () => {
    for (const fileName of [
      "_runner-utils.mjs",
      "pre-tool-use.mjs",
      "post-tool-batch.mjs",
      "stop.mjs",
      "user-prompt-submit.mjs",
    ]) {
      expect(existsSync(path.join(out, "hooks", fileName))).toBe(true);
    }
  });

  it("includes Claude Code skill surfaces with the shared engine", () => {
    expect(existsSync(path.join(out, "skills/lumin-repo-lens-lab/SKILL.md"))).toBe(
      true,
    );
    expect(
      existsSync(
        path.join(out, "skills/lumin-repo-lens-lab/scripts/audit-repo.mjs"),
      ),
    ).toBe(true);
    expect(
      existsSync(
        path.join(
          out,
          "skills/lumin-repo-lens-lab/_engine/producers/audit-repo.mjs",
        ),
      ),
    ).toBe(true);
    expect(
      existsSync(path.join(out, "skills/lumin-repo-lens-lab-write-gate/SKILL.md")),
    ).toBe(true);
    expect(
      existsSync(path.join(out, "skills/lumin-repo-lens-lab-canon/SKILL.md")),
    ).toBe(true);
  });

  it("excludes the Codex wrapper by default to avoid Claude Code surface collision", () => {
    expect(existsSync(path.join(out, "skills/lumin-repo-lens-lab-codex"))).toBe(
      false,
    );
    expect(
      existsSync(path.join(out, "skills/lumin-repo-lens-lab-codex/SKILL.md")),
    ).toBe(false);
  });

  it("targets packaged plugin-root skill paths from the default command", () => {
    const command = readFileSync(
      path.join(out, "commands/lumin-repo-lens-lab.md"),
      "utf8",
    );

    expect(command).toContain(
      "${CLAUDE_PLUGIN_ROOT}/skills/lumin-repo-lens-lab/SKILL.md",
    );
    expect(command).toContain(
      "${CLAUDE_PLUGIN_ROOT}/skills/lumin-repo-lens-lab/references/command-routing.md",
    );
  });

  it("resolves every command plugin-root reference inside the packaged root", () => {
    const unresolved = [];
    for (const fileName of readdirSync(path.join(out, "commands"))
      .filter((name) => name.endsWith(".md"))
      .sort()) {
      const text = readFileSync(path.join(out, "commands", fileName), "utf8");
      for (const ref of pluginRefsFromCommand(text)) {
        if (!existsSync(path.join(out, ref)))
          unresolved.push(`${fileName}: ${ref}`);
      }
    }

    expect(unresolved).toEqual([]);
  });

  it("delegates every command to the expected packaged skill surface", () => {
    const wrongTargets = [];
    for (const fileName of readdirSync(path.join(out, "commands"))
      .filter((name) => name.endsWith(".md"))
      .sort()) {
      const commandName = fileName.replace(/\.md$/, "");
      const expectedSkill = COMMAND_SKILL_TARGETS[commandName];
      if (!expectedSkill) continue;

      const text = readFileSync(path.join(out, "commands", fileName), "utf8");
      const refs = pluginRefsFromCommand(text);
      if (!refs.includes(expectedSkill)) {
        wrongTargets.push(
          `${fileName}: expected ${expectedSkill}, got ${refs.join(", ")}`,
        );
      }
    }

    expect(wrongTargets).toEqual([]);
  });

  it("carries versioned plugin metadata plus the skill distribution marker", () => {
    const plugin = JSON.parse(
      readFileSync(path.join(out, ".claude-plugin/plugin.json"), "utf8"),
    );
    const skillPkg = JSON.parse(
      readFileSync(
        path.join(out, "skills/lumin-repo-lens-lab/package.json"),
        "utf8",
      ),
    );

    expect(plugin.name).toBe("lumin-repo-lens-lab");
    expect(plugin.description).toContain("repo structure lens");
    expect(plugin.version).toBe(skillPkg.version);
    expect(skillPkg.luminRepoLens?.distribution).toBe("skill");
  });

  it("README names the install root and warns against installing skills alone", () => {
    const packageReadme = readFileSync(
      path.join(out, "README.plugin-package.md"),
      "utf8",
    );

    expect(packageReadme).toContain(
      "Install this directory as the Claude Code plugin root",
    );
    expect(packageReadme).toContain("Do not install `skills/` alone");
    expect(packageReadme).toContain(
      "Slash command delegators resolve through `${CLAUDE_PLUGIN_ROOT}`",
    );
  });

  it("runs the plugin-root smoke check after staging", () => {
    expect(build.stdout).toContain(
      "[build-plugin-package] plugin-root smoke passed",
    );
  });

  it("removes stale legacy plugin output beside the current package", () => {
    expect(existsSync(legacyOut)).toBe(false);
    expect(build.stdout).toContain("removed legacy output");
  });

  it("includes the Codex wrapper only when explicitly requested", () => {
    const outWithCodex = path.join(tmp, "lumin-repo-lens-lab-plugin-with-codex");
    mkdirSync(path.dirname(outWithCodex), { recursive: true });

    const buildWithCodex = spawnSync(
      NODE,
      [buildPlugin, "--out", outWithCodex, "--include-codex"],
      {
        cwd: fixtureRoot,
        encoding: "utf8",
      },
    );

    expect(`${buildWithCodex.stdout}\n${buildWithCodex.stderr}`).toBeDefined();
    expect(buildWithCodex.status).toBe(0);
    expect(
      existsSync(
        path.join(outWithCodex, "skills/lumin-repo-lens-lab-codex/SKILL.md"),
      ),
    ).toBe(true);
  });
});
