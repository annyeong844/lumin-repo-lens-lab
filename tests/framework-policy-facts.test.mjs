import { describe, expect, it } from "vitest";

import { shouldCollectHonoRouteFactsForPackages } from "../_lib/classify-policies.mjs";
import { collectHonoRouteRegistrations } from "../_lib/framework-policy-facts.mjs";
import { createTempRepoFixture } from "./_helpers/temp-repo-fixture.mjs";

function collect(root, files) {
  return collectHonoRouteRegistrations({ root, files });
}

describe("framework policy facts", () => {
  it("T1. collects imported Hono handlers from app.get handlerRefs array", () => {
    const fixture = createTempRepoFixture({
      prefix: "fx-vitest-hono-facts-",
      packageJson: { private: true },
    });

    try {
      fixture.write(
        "src/server.ts",
        [
          "import { Hono } from 'hono';",
          "import { auth } from './middleware';",
          "import { handler } from './handlers';",
          "const app = new Hono();",
          "app.get('/x', auth, handler);",
          "",
        ].join("\n"),
      );
      fixture.write(
        "src/middleware.ts",
        "export function auth(c) { return c.next(); }\n",
      );
      fixture.write(
        "src/handlers.ts",
        'export function handler(c) { return c.text("ok"); }\n',
      );

      expect(
        collect(fixture.root, [
          "src/server.ts",
          "src/middleware.ts",
          "src/handlers.ts",
        ]),
      ).toEqual([
        {
          file: "src/server.ts",
          callee: "app.get",
          route: "/x",
          handlerRefs: [
            { file: "src/middleware.ts", exportName: "auth" },
            { file: "src/handlers.ts", exportName: "handler" },
          ],
        },
      ]);
    } finally {
      fixture.cleanup();
    }
  });

  it("T2. collects app.use and app.route references", () => {
    const fixture = createTempRepoFixture({
      prefix: "fx-vitest-hono-facts-",
      packageJson: { private: true },
    });

    try {
      fixture.write(
        "src/server.ts",
        [
          "import { Hono } from 'hono';",
          "import { auth } from './middleware';",
          "import { apiRoutes } from './api';",
          "const app = new Hono();",
          "app.use('/x', auth);",
          "app.route('/api', apiRoutes);",
          "",
        ].join("\n"),
      );
      fixture.write(
        "src/middleware.ts",
        "export const auth = (c, next) => next();\n",
      );
      fixture.write("src/api.ts", "export const apiRoutes = new Hono();\n");

      expect(
        collect(fixture.root, [
          "src/server.ts",
          "src/middleware.ts",
          "src/api.ts",
        ]),
      ).toEqual([
        {
          file: "src/server.ts",
          callee: "app.use",
          route: "/x",
          handlerRefs: [{ file: "src/middleware.ts", exportName: "auth" }],
        },
        {
          file: "src/server.ts",
          callee: "app.route",
          route: "/api",
          handlerRefs: [{ file: "src/api.ts", exportName: "apiRoutes" }],
        },
      ]);
    } finally {
      fixture.cleanup();
    }
  });

  it("T3. collects local exported handlers and skips dynamic handler expressions", () => {
    const fixture = createTempRepoFixture({
      prefix: "fx-vitest-hono-facts-",
      packageJson: { private: true },
    });

    try {
      fixture.write(
        "src/server.ts",
        [
          "import { Hono } from 'hono';",
          "const app = new Hono();",
          'export function localHandler(c) { return c.text("local"); }',
          "app.post('/local', localHandler);",
          "app.get('/dynamic', makeHandler());",
          "",
        ].join("\n"),
      );

      expect(collect(fixture.root, ["src/server.ts"])).toEqual([
        {
          file: "src/server.ts",
          callee: "app.post",
          route: "/local",
          handlerRefs: [{ file: "src/server.ts", exportName: "localHandler" }],
        },
      ]);
    } finally {
      fixture.cleanup();
    }
  });

  it("T4. Hono route fact collection is gated by package-scoped Hono dependency", () => {
    expect(
      shouldCollectHonoRouteFactsForPackages([
        { relRoot: ".", packageJson: { dependencies: { next: "^15.0.0" } } },
      ]),
    ).toBe(false);
    expect(
      shouldCollectHonoRouteFactsForPackages([
        { relRoot: ".", packageJson: { dependencies: { hono: "^4.0.0" } } },
      ]),
    ).toBe(true);
    expect(
      shouldCollectHonoRouteFactsForPackages([
        { relRoot: ".", packageJson: { devDependencies: { hono: "^4.0.0" } } },
      ]),
    ).toBe(true);
  });
});
