import { execFileSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

import { parseMdxImportConsumers } from "../_lib/mdx-consumers.mjs";
import { createTempRepoFixture } from "./_helpers/temp-repo-fixture.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(__dirname, "..");

function runSymbolGraph(fixture) {
  execFileSync(
    process.execPath,
    [
      path.join(REPO_ROOT, "build-symbol-graph.mjs"),
      "--root",
      fixture.root,
      "--output",
      fixture.output,
    ],
    {
      cwd: REPO_ROOT,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    },
  );
  return fixture.readJson("symbols.json", { from: "output" });
}

describe("MDX consumers", () => {
  it("MX-1. parses MDX imports and ignores fenced example imports", () => {
    const src = [
      "import DefaultCard, { UsedByMdx as Card, type Props } from '../src/card';",
      "import * as Widgets from '../src/widgets';",
      "import DefaultWidget, * as WidgetNamespace from '../src/widget-namespace';",
      "```tsx",
      "import { ExampleOnly } from '../src/example';",
      "```",
      "```md",
      "```tsx",
      "import { NestedExampleOnly } from '../src/nested-example';",
      "```",
      "",
      "<Card />",
    ].join("\n");

    const imports = parseMdxImportConsumers(src, "content/page.mdx");
    const names = imports
      .map((entry) => `${entry.fromSpec}:${entry.name}:${entry.kind}`)
      .sort();

    expect(names).toContain("../src/card:UsedByMdx:import");
    expect(names).toContain("../src/card:default:default");
    expect(names).toContain("../src/widgets:*:namespace");
    expect(names).toContain("../src/widget-namespace:default:default");
    expect(names).toContain("../src/widget-namespace:*:namespace");
    expect(
      names.some((name) => name.includes("../src/example")),
      JSON.stringify(imports),
    ).toBe(false);
    expect(
      names.some((name) => name.includes("../src/nested-example")),
      JSON.stringify(imports),
    ).toBe(false);
  });

  it("MX-2. contributes MDX fan-in evidence without protecting fenced imports", () => {
    const fixture = createTempRepoFixture({
      prefix: "vitest-mdx-consumer-graph-",
      packageJson: { name: "mdx-fixture", type: "module" },
      outputDirName: "out",
    });
    try {
      fixture.write(
        "src/card.tsx",
        [
          "export function UsedByMdx() { return null; }",
          "export const Unused = 1;",
          "export default function DefaultCard() { return null; }",
          "",
        ].join("\n"),
      );
      fixture.write(
        "src/widgets.tsx",
        [
          "export function UsedByNamespace() { return null; }",
          "export default function DefaultWidget() { return null; }",
          "",
        ].join("\n"),
      );
      fixture.write(
        "content/page.mdx",
        [
          "import DefaultCard, { UsedByMdx as Card } from '../src/card';",
          "import DefaultWidget, * as Widgets from '../src/widgets';",
          "```md",
          "```tsx",
          "import { Unused } from '../src/card';",
          "```",
          "",
          "<DefaultCard />",
          "<Card />",
          "",
        ].join("\n"),
      );

      const symbols = runSymbolGraph(fixture);
      const dead = new Set(
        (symbols.deadProdList ?? []).map(
          (entry) => `${entry.file}::${entry.symbol}`,
        ),
      );

      expect(dead).not.toContain("src/card.tsx::UsedByMdx");
      expect(dead).not.toContain("src/card.tsx::default");
      expect(dead).toContain("src/card.tsx::Unused");
      expect(dead).not.toContain("src/widgets.tsx::default");
      expect(dead).not.toContain("src/widgets.tsx::UsedByNamespace");
      expect(symbols.fanInByIdentity?.["src/card.tsx::Unused"]).toBe(0);
      expect(symbols.fanInByIdentity?.["src/card.tsx::UsedByMdx"]).toBe(1);
      expect(symbols.uses?.mdxConsumers).toBe(4);
    } finally {
      fixture.cleanup();
    }
  });
});
