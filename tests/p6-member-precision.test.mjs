import { execFileSync } from "node:child_process";
import {
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import path from "node:path";
import { tmpdir } from "node:os";
import { fileURLToPath } from "node:url";
import { expect, it } from "vitest";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const DIR = path.resolve(__dirname, "..");
const NODE = process.execPath;
const BUILD = path.join(DIR, "build-symbol-graph.mjs");

function assert(label, ok, detail = "") {
  it(label, () => {
    expect(ok, detail).toBeTruthy();
  });
}

function write(root, rel, content) {
  const full = path.join(root, rel);
  mkdirSync(path.dirname(full), { recursive: true });
  writeFileSync(full, content);
}

function runSymbols(root) {
  const out = mkdtempSync(path.join(tmpdir(), "p6-member-out-"));
  execFileSync(NODE, [BUILD, "--root", root, "--output", out], {
    cwd: DIR,
    stdio: ["ignore", "pipe", "pipe"],
  });
  return {
    out,
    symbols: JSON.parse(readFileSync(path.join(out, "symbols.json"), "utf8")),
  };
}

function deadSymbol(symbols, name) {
  return (symbols.deadProdList ?? []).find((d) => d.symbol === name);
}

{
  const fx = mkdtempSync(path.join(tmpdir(), "p6-member-ns-"));
  let out;
  try {
    write(
      fx,
      "package.json",
      JSON.stringify({ name: "p6-member-ns", type: "module" }),
    );
    write(
      fx,
      "src/mod.ts",
      `export function used() { return 1; }\n` +
        `export function unused() { return 2; }\n`,
    );
    write(
      fx,
      "src/consumer.ts",
      `import * as mod from './mod';\n` +
        `export function run() { return mod.used(); }\n`,
    );

    const result = runSymbols(fx);
    out = result.out;
    const symbols = result.symbols;

    assert(
      "P6M-1a. namespace direct member protects only the called export",
      symbols.fanInByIdentity?.["src/mod.ts::used"] === 1 &&
        !deadSymbol(symbols, "used"),
      JSON.stringify({
        fanIn: symbols.fanInByIdentity?.["src/mod.ts::used"],
        dead: deadSymbol(symbols, "used"),
      }),
    );
    assert(
      "P6M-1b. unrelated namespace sibling remains a concrete dead candidate",
      !!deadSymbol(symbols, "unused") &&
        deadSymbol(symbols, "unused")?.namespaceShadowed === false,
      JSON.stringify(deadSymbol(symbols, "unused")),
    );
  } finally {
    rmSync(fx, { recursive: true, force: true });
    if (out) rmSync(out, { recursive: true, force: true });
  }
}

{
  const fx = mkdtempSync(path.join(tmpdir(), "p6-member-dyn-var-"));
  let out;
  try {
    write(
      fx,
      "package.json",
      JSON.stringify({ name: "p6-member-dyn-var", type: "module" }),
    );
    write(
      fx,
      "src/mod.ts",
      `export function loaded() { return 1; }\n` +
        `export function unusedDynamic() { return 2; }\n`,
    );
    write(
      fx,
      "src/consumer.ts",
      `export async function run() {\n` +
        `  const mod = await import('./mod');\n` +
        `  return mod.loaded();\n` +
        `}\n`,
    );

    const result = runSymbols(fx);
    out = result.out;
    const symbols = result.symbols;

    assert(
      "P6M-2a. await import binding direct member protects the called export",
      symbols.fanInByIdentity?.["src/mod.ts::loaded"] === 1 &&
        !deadSymbol(symbols, "loaded"),
      JSON.stringify({
        fanIn: symbols.fanInByIdentity?.["src/mod.ts::loaded"],
        dead: deadSymbol(symbols, "loaded"),
      }),
    );
    assert(
      "P6M-2b. await import direct member does not blanket-protect siblings",
      !!deadSymbol(symbols, "unusedDynamic") &&
        deadSymbol(symbols, "unusedDynamic")?.namespaceShadowed === false,
      JSON.stringify(deadSymbol(symbols, "unusedDynamic")),
    );
  } finally {
    rmSync(fx, { recursive: true, force: true });
    if (out) rmSync(out, { recursive: true, force: true });
  }
}

{
  const fx = mkdtempSync(path.join(tmpdir(), "p6-member-dyn-then-"));
  let out;
  try {
    write(
      fx,
      "package.json",
      JSON.stringify({ name: "p6-member-dyn-then", type: "module" }),
    );
    write(
      fx,
      "src/mod.ts",
      `export function thenLoaded() { return 1; }\n` +
        `export function thenUnused() { return 2; }\n`,
    );
    write(
      fx,
      "src/consumer.ts",
      `export function run() {\n` +
        `  return import('./mod').then((m) => m.thenLoaded());\n` +
        `}\n`,
    );

    const result = runSymbols(fx);
    out = result.out;
    const symbols = result.symbols;

    assert(
      "P6M-3a. import().then callback member protects the called export",
      symbols.fanInByIdentity?.["src/mod.ts::thenLoaded"] === 1 &&
        !deadSymbol(symbols, "thenLoaded"),
      JSON.stringify({
        fanIn: symbols.fanInByIdentity?.["src/mod.ts::thenLoaded"],
        dead: deadSymbol(symbols, "thenLoaded"),
      }),
    );
    assert(
      "P6M-3b. import().then direct member does not blanket-protect siblings",
      !!deadSymbol(symbols, "thenUnused") &&
        deadSymbol(symbols, "thenUnused")?.namespaceShadowed === false,
      JSON.stringify(deadSymbol(symbols, "thenUnused")),
    );
  } finally {
    rmSync(fx, { recursive: true, force: true });
    if (out) rmSync(out, { recursive: true, force: true });
  }
}

{
  const fx = mkdtempSync(path.join(tmpdir(), "p6-member-degraded-"));
  let out;
  try {
    write(
      fx,
      "package.json",
      JSON.stringify({ name: "p6-member-degraded", type: "module" }),
    );
    write(
      fx,
      "src/mod.ts",
      `export function maybe() { return 1; }\n` +
        `export function other() { return 2; }\n`,
    );
    write(
      fx,
      "src/consumer.ts",
      `import * as mod from './mod';\n` +
        `const f = mod.maybe;\n` +
        `const run = () => f();\n` +
        `run();\n`,
    );

    const result = runSymbols(fx);
    out = result.out;
    const symbols = result.symbols;

    assert(
      "P6M-4. degraded namespace alias keeps the conservative whole-file shadow",
      symbols.deadTotal === 2 &&
        symbols.trulyDead === 0 &&
        !deadSymbol(symbols, "maybe") &&
        !deadSymbol(symbols, "other"),
      JSON.stringify({
        deadTotal: symbols.deadTotal,
        trulyDead: symbols.trulyDead,
        deadProdList: symbols.deadProdList,
      }),
    );
  } finally {
    rmSync(fx, { recursive: true, force: true });
    if (out) rmSync(out, { recursive: true, force: true });
  }
}

{
  const fx = mkdtempSync(path.join(tmpdir(), "p6-member-dyn-shadow-"));
  let out;
  try {
    write(
      fx,
      "package.json",
      JSON.stringify({ name: "p6-member-dyn-shadow", type: "module" }),
    );
    write(
      fx,
      "src/mod.ts",
      `export function safe() { return 1; }\n` +
        `export function colliding() { return 2; }\n` +
        `export function unused() { return 3; }\n`,
    );
    write(
      fx,
      "src/other.ts",
      `export function colliding() { return 4; }\n` +
        `export function otherUnused() { return 5; }\n`,
    );
    write(
      fx,
      "src/consumer.ts",
      `export async function run(flag) {\n` +
        `  const mod = await import('./mod');\n` +
        `  if (flag) {\n` +
        `    const mod = await import('./other');\n` +
        `    return mod.colliding();\n` +
        `  }\n` +
        `  return mod.safe();\n` +
        `}\n`,
    );

    const result = runSymbols(fx);
    out = result.out;
    const symbols = result.symbols;

    assert(
      "P6M-5a. shadowed dynamic binding keeps outer member attribution lexical",
      symbols.fanInByIdentity?.["src/mod.ts::safe"] === 1 &&
        symbols.fanInByIdentity?.["src/mod.ts::colliding"] === 0,
      JSON.stringify({
        safe: symbols.fanInByIdentity?.["src/mod.ts::safe"],
        colliding: symbols.fanInByIdentity?.["src/mod.ts::colliding"],
      }),
    );
    assert(
      "P6M-5b. shadowed inner dynamic binding attributes to its own module",
      symbols.fanInByIdentity?.["src/other.ts::colliding"] === 1 &&
        symbols.fanInByIdentity?.["src/other.ts::otherUnused"] === 0,
      JSON.stringify({
        colliding: symbols.fanInByIdentity?.["src/other.ts::colliding"],
        otherUnused: symbols.fanInByIdentity?.["src/other.ts::otherUnused"],
      }),
    );
    assert(
      "P6M-5c. shadowed dynamic binding does not hide unrelated dead exports",
      !!deadSymbol(symbols, "unused") && !!deadSymbol(symbols, "otherUnused"),
      JSON.stringify(symbols.deadProdList),
    );
  } finally {
    rmSync(fx, { recursive: true, force: true });
    if (out) rmSync(out, { recursive: true, force: true });
  }
}

{
  const fx = mkdtempSync(path.join(tmpdir(), "p6-member-ns-shadow-"));
  let out;
  try {
    write(
      fx,
      "package.json",
      JSON.stringify({ name: "p6-member-ns-shadow", type: "module" }),
    );
    write(
      fx,
      "src/mod.ts",
      `export function real() { return 1; }\n` +
        `export function shadowOnly() { return 2; }\n`,
    );
    write(
      fx,
      "src/consumer.ts",
      `import * as mod from './mod';\n` +
        `function local(mod) { return mod.shadowOnly(); }\n` +
        `export function run() { return mod.real() + local({ shadowOnly: () => 0 }); }\n`,
    );

    const result = runSymbols(fx);
    out = result.out;
    const symbols = result.symbols;

    assert(
      "P6M-6a. namespace parameter shadow does not steal module attribution",
      symbols.fanInByIdentity?.["src/mod.ts::real"] === 1 &&
        symbols.fanInByIdentity?.["src/mod.ts::shadowOnly"] === 0,
      JSON.stringify({
        real: symbols.fanInByIdentity?.["src/mod.ts::real"],
        shadowOnly: symbols.fanInByIdentity?.["src/mod.ts::shadowOnly"],
      }),
    );
    assert(
      "P6M-6b. namespace shadowed-only export remains a concrete dead candidate",
      !!deadSymbol(symbols, "shadowOnly") &&
        deadSymbol(symbols, "shadowOnly")?.namespaceShadowed === false,
      JSON.stringify(deadSymbol(symbols, "shadowOnly")),
    );
  } finally {
    rmSync(fx, { recursive: true, force: true });
    if (out) rmSync(out, { recursive: true, force: true });
  }
}
