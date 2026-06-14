import path from "node:path";

import { describe, expect, it } from "vitest";

import { discoverScopedTsconfigPaths } from "../_lib/tsconfig-paths.mjs";
import { createTempRepoFixture } from "./_helpers/temp-repo-fixture.mjs";

function slash(value) {
  return String(value).replace(/\\/g, "/");
}

function findScopeEntry(entries, scopeSuffix, key = "@/*") {
  return entries.find(
    (entry) => entry.key === key && slash(entry.scopeDir).endsWith(scopeSuffix),
  );
}

function withFixture(prefix, fn) {
  const fixture = createTempRepoFixture({ prefix });
  try {
    return fn(fixture);
  } finally {
    fixture.cleanup();
  }
}

describe("JSONC tsconfig path discovery edge cases", () => {
  it("parses $schema URLs without stripping string-contained line-comment markers", () => {
    withFixture("fx-vitest-jsonc-schema-", (fixture) => {
      fixture.writeJson("apps/a/tsconfig.json", {
        $schema: "https://json.schemastore.org/tsconfig",
        compilerOptions: { baseUrl: ".", paths: { "@/*": ["./*"] } },
      });

      const entries = discoverScopedTsconfigPaths(fixture.root);

      expect(findScopeEntry(entries, "apps/a")).toBeTruthy();
    });
  });

  it("parses JSONC line comments, block comments, and trailing line comments", () => {
    withFixture("fx-vitest-jsonc-comments-", (fixture) => {
      fixture.write(
        "apps/a/tsconfig.json",
        `{
  // This is a line comment at the start of a line
  "compilerOptions": {
    /* block comment
       spanning lines */
    "baseUrl": ".",
    "paths": {
      "@/*": ["./*"]  // trailing line comment
    }
  }
}
`,
      );

      const entries = discoverScopedTsconfigPaths(fixture.root);

      expect(findScopeEntry(entries, "apps/a")).toBeTruthy();
    });
  });

  it("tolerates trailing commas in tsconfig objects, arrays, and paths maps", () => {
    withFixture("fx-vitest-jsonc-trailing-", (fixture) => {
      fixture.write(
        "apps/a/tsconfig.json",
        `{
  "compilerOptions": {
    "baseUrl": ".",
    "paths": {
      "@/*": ["./*"],
      "@lib/*": ["./lib/*"],
    },
  },
}
`,
      );

      const entries = discoverScopedTsconfigPaths(fixture.root);

      expect(findScopeEntry(entries, "apps/a")).toBeTruthy();
      expect(findScopeEntry(entries, "apps/a", "@lib/*")).toBeTruthy();
    });
  });

  it("does not treat block-comment-looking text inside strings as comments", () => {
    withFixture("fx-vitest-jsonc-stringblock-", (fixture) => {
      fixture.write(
        "apps/a/tsconfig.json",
        `{
  "compilerOptions": {
    "baseUrl": ".",
    "paths": {
      "@/*": ["./*"]
    }
  },
  "exclude": ["/* ignore */ generated"]
}
`,
      );

      const entries = discoverScopedTsconfigPaths(fixture.root);

      expect(findScopeEntry(entries, "apps/a")).toBeTruthy();
    });
  });

  it("parses BOM-prefixed tsconfig.json files", () => {
    withFixture("fx-vitest-jsonc-bom-", (fixture) => {
      const body = JSON.stringify({
        compilerOptions: { baseUrl: ".", paths: { "@/*": ["./*"] } },
      });
      fixture.write("apps/a/tsconfig.json", `\uFEFF${body}`);

      const entries = discoverScopedTsconfigPaths(fixture.root);

      expect(findScopeEntry(entries, "apps/a")).toBeTruthy();
    });
  });

  it("keeps local paths when extends points at a missing package", () => {
    withFixture("fx-vitest-jsonc-duyet-", (fixture) => {
      fixture.mkdir("apps/agents/components");
      fixture.mkdir("apps/agents/app");
      fixture.mkdir("apps/admin");
      fixture.mkdir("node_modules/@ghost");

      const tsconfig = `{
  "$schema": "https://json.schemastore.org/tsconfig",
  "extends": "@ghost/tsconfig/vite.json",
  "compilerOptions": {
    "baseUrl": ".",
    "paths": { "@/*": ["./*"] }
  }
}
`;

      fixture.write("apps/agents/tsconfig.json", tsconfig);
      fixture.write("apps/admin/tsconfig.json", tsconfig);

      const entries = discoverScopedTsconfigPaths(fixture.root);

      expect(findScopeEntry(entries, "apps/agents")).toBeTruthy();
      expect(findScopeEntry(entries, "apps/admin")).toBeTruthy();
    });
  });

  it("discovers every duyet-shaped app tsconfig instead of only a subset", () => {
    withFixture("fx-vitest-jsonc-duyet-apps-", (fixture) => {
      const tsconfig = `{
  "$schema": "https://json.schemastore.org/tsconfig",
  "extends": "@ghost/tsconfig/vite.json",
  "compilerOptions": {
    "baseUrl": ".",
    "paths": { "@/*": ["./*"] }
  }
}
`;

      for (let index = 1; index <= 11; index++) {
        fixture.write(`apps/app-${index}/tsconfig.json`, tsconfig);
      }

      const entries = discoverScopedTsconfigPaths(fixture.root);
      const appEntries = entries.filter(
        (entry) =>
          entry.key === "@/*" && /\/apps\/app-\d+$/.test(slash(entry.scopeDir)),
      );

      expect(appEntries).toHaveLength(11);
    });
  });
});
