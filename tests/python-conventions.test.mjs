import { execFileSync } from "node:child_process";
import path from "node:path";

import { describe, expect, it } from "vitest";

import { createTempRepoFixture } from "./_helpers/temp-repo-fixture.mjs";

const ROOT = path.resolve(import.meta.dirname, "..");
const PYTHON_CONVENTIONS_TIMEOUT_MS = 30_000;

function hasPython3() {
  try {
    execFileSync("python3", ["-c", "pass"], { stdio: "ignore" });
    return true;
  } catch {
    return false;
  }
}

const describePython = hasPython3() ? describe : describe.skip;

function buildSymbols(fixture) {
  execFileSync(
    process.execPath,
    [
      "build-symbol-graph.mjs",
      "--root",
      fixture.root,
      "--output",
      fixture.output,
    ],
    {
      cwd: ROOT,
      stdio: ["ignore", "pipe", "pipe"],
    },
  );
  return fixture.readJson("symbols.json", { from: "output" });
}

function deadSymbolsForFile(symbols, fileSuffix) {
  return new Set(
    (symbols.deadProdList ?? [])
      .filter((entry) => entry.file.endsWith(fileSuffix))
      .map((entry) => entry.symbol),
  );
}

describePython("Python convention support", () => {
  it(
    "resolves package self-reference imports without blanket liveness",
    () => {
      const fixture = createTempRepoFixture({
        prefix: "fxpyselfref",
        packageJson: { name: "fxpyselfref", private: true },
      });
      try {
        const packageName = path.basename(fixture.root);

        fixture.write("__init__.py", "");
        fixture.write("agents/__init__.py", "");
        fixture.write("consumers/__init__.py", "");
        fixture.write(
          "agents/loader.py",
          'def load_agent() -> str:\n    return "x"\n',
        );
        fixture.write(
          "consumers/app.py",
          `from ${packageName}.agents.loader import load_agent\n` +
            "\n" +
            "def main():\n" +
            "    return load_agent()\n",
        );

        const deadNames = new Set(
          (buildSymbols(fixture).deadProdList ?? []).map(
            (entry) => entry.symbol,
          ),
        );

        expect(deadNames.has("load_agent")).toBe(false);
        expect(deadNames.has("main")).toBe(true);
      } finally {
        fixture.cleanup();
      }
    },
    PYTHON_CONVENTIONS_TIMEOUT_MS,
  );

  it(
    "keeps __all__ filtering precise for public and private names",
    () => {
      const fixture = createTempRepoFixture({
        prefix: "fxpydunderall-",
        packageJson: { name: "fxpydunderall", private: true },
      });
      try {
        fixture.write("__init__.py", "");
        fixture.write("dummy.py", "x = 1\n");
        fixture.write(
          "module.py",
          '__all__ = ["Foo"]\n' +
            "\n" +
            "class Foo:\n" +
            "    def method(self):\n" +
            "        return self._helper()\n" +
            "\n" +
            "def _helper():\n" +
            "    return 1\n" +
            "\n" +
            "def internal_util():\n" +
            "    return 2\n",
        );

        const deadNames = deadSymbolsForFile(
          buildSymbols(fixture),
          "module.py",
        );

        expect(deadNames.has("__all__")).toBe(false);
        expect(deadNames.has("Foo")).toBe(true);
        expect(deadNames.has("_helper")).toBe(false);
        expect(deadNames.has("internal_util")).toBe(false);
      } finally {
        fixture.cleanup();
      }
    },
    PYTHON_CONVENTIONS_TIMEOUT_MS,
  );

  it(
    "treats framework decorators as dispatch evidence without muting plain functions",
    () => {
      const fixture = createTempRepoFixture({
        prefix: "fxpytyper-",
        packageJson: { name: "fxpytyper", private: true },
      });
      try {
        fixture.write("__init__.py", "");
        fixture.write("dummy.py", "x = 1\n");
        fixture.write(
          "cli.py",
          "import typer\n" +
            "app = typer.Typer()\n" +
            "\n" +
            "@app.command()\n" +
            "def subcommand_a():\n" +
            "    return 1\n" +
            "\n" +
            '@app.command(name="list")\n' +
            "def list_items():\n" +
            "    return 2\n" +
            "\n" +
            "@app.callback()\n" +
            "def callback_entry():\n" +
            "    return 3\n" +
            "\n" +
            "def actually_unused():\n" +
            "    return 4\n",
        );

        const deadNames = deadSymbolsForFile(buildSymbols(fixture), "cli.py");

        expect(deadNames.has("subcommand_a")).toBe(false);
        expect(deadNames.has("list_items")).toBe(false);
        expect(deadNames.has("callback_entry")).toBe(false);
        expect(deadNames.has("actually_unused")).toBe(true);
      } finally {
        fixture.cleanup();
      }
    },
    PYTHON_CONVENTIONS_TIMEOUT_MS,
  );

  it(
    "excludes dunder runtime hooks while keeping ordinary functions eligible",
    () => {
      const fixture = createTempRepoFixture({
        prefix: "fxpydunder-",
        packageJson: { name: "fxpydunder", private: true },
      });
      try {
        fixture.write(
          "__init__.py",
          "# Module-level __getattr__ is a lazy-loading hook\n" +
            "def __getattr__(name):\n" +
            "    raise AttributeError(name)\n" +
            "\n" +
            "def __dir__():\n" +
            "    return []\n" +
            "\n" +
            "def regular_fn():\n" +
            "    return 1\n",
        );
        fixture.write("dummy.py", "x = 1\n");

        const deadNames = deadSymbolsForFile(
          buildSymbols(fixture),
          "__init__.py",
        );

        expect(deadNames.has("__getattr__")).toBe(false);
        expect(deadNames.has("__dir__")).toBe(false);
        expect(deadNames.has("regular_fn")).toBe(true);
      } finally {
        fixture.cleanup();
      }
    },
    PYTHON_CONVENTIONS_TIMEOUT_MS,
  );
});
