import { readFileSync } from 'node:fs';
import path from 'node:path';

export const ESLINT_CONFIG_PATTERN = /^\.eslintrc.*$|^eslint\.config\.[mc]?[jt]s$/;
export const OXLINT_CONFIG_PATTERN = /^\.oxlintrc(?:\.[^.]+)*\.json$/;

const NORMALIZED_RULES = new Set([
  'no-restricted-syntax',
  'no-restricted-imports',
  'no-restricted-paths',
  'no-explicit-any',
]);

function normalizedRule(ruleName) {
  if (typeof ruleName !== 'string') return null;
  if (ruleName.startsWith('boundaries/')) return 'eslint-plugin-boundaries';
  const leaf = ruleName.slice(ruleName.lastIndexOf('/') + 1);
  return NORMALIZED_RULES.has(leaf) ? leaf : null;
}

function oxlintRuleEnabled(setting) {
  const severity = Array.isArray(setting) ? setting[0] : setting;
  return severity !== 0 && severity !== false && severity !== null &&
    severity !== 'off' && severity !== 'allow';
}

function lintCommands(packageScripts) {
  if (!packageScripts || typeof packageScripts !== 'object') return [];
  return Object.entries(packageScripts)
    .filter(([scriptName, command]) =>
      /^lint(?::|$)/.test(scriptName) && typeof command === 'string')
    .map(([scriptName, command]) => {
      const tools = [];
      if (/\beslint\b/.test(command)) tools.push('eslint');
      if (/\boxlint\b/.test(command)) tools.push('oxlint');
      const delegated = /\b(?:npm|pnpm|yarn|bun)\s+(?:run\s+)?lint(?::[^\s;&|]+)?\b/.test(command);
      return {
        scriptName,
        command,
        tools,
        status: tools.length > 0 || delegated ? 'supported' : 'unsupported',
      };
    });
}

export function collectLintEnforcement({
  root,
  eslintConfigs,
  oxlintConfigs,
  packageScripts,
}) {
  const boundaries = [];
  const boundaryKeys = new Set();
  const configs = [];
  const diagnostics = [];

  function addBoundary(rule, file, tool) {
    const key = `${rule}\0${file}`;
    if (boundaryKeys.has(key)) return;
    boundaryKeys.add(key);
    boundaries.push(tool ? { rule, file, tool } : { rule, file });
  }

  for (const file of eslintConfigs) {
    try {
      const content = readFileSync(path.join(root, file), 'utf8');
      for (const rule of NORMALIZED_RULES) {
        if (content.includes(rule)) addBoundary(rule, file);
      }
      if (content.includes('boundaries/')) addBoundary('eslint-plugin-boundaries', file);
      configs.push({ tool: 'eslint', file, status: 'scanned', parser: 'rule-name-text-scan' });
    } catch {
      configs.push({ tool: 'eslint', file, status: 'invalid', parser: 'rule-name-text-scan' });
      diagnostics.push({ kind: 'lint-config-unreadable', tool: 'eslint', file });
    }
  }

  for (const file of oxlintConfigs) {
    let config;
    try {
      config = JSON.parse(readFileSync(path.join(root, file), 'utf8'));
    } catch {
      configs.push({ tool: 'oxlint', file, status: 'invalid', parser: 'json-rules' });
      diagnostics.push({ kind: 'lint-config-invalid-json', tool: 'oxlint', file });
      continue;
    }
    if (!config || typeof config !== 'object' || Array.isArray(config)) {
      configs.push({ tool: 'oxlint', file, status: 'invalid', parser: 'json-rules' });
      diagnostics.push({ kind: 'lint-config-invalid-shape', tool: 'oxlint', file });
      continue;
    }

    const ruleSets = [
      config.rules,
      ...(Array.isArray(config.overrides)
        ? config.overrides.map((override) => override?.rules)
        : []),
    ];
    for (const rules of ruleSets) {
      if (!rules || typeof rules !== 'object' || Array.isArray(rules)) continue;
      for (const [ruleName, setting] of Object.entries(rules)) {
        const rule = normalizedRule(ruleName);
        if (rule && oxlintRuleEnabled(setting)) addBoundary(rule, file, 'oxlint');
      }
    }
    configs.push({ tool: 'oxlint', file, status: 'scanned', parser: 'json-rules' });
  }

  const commands = lintCommands(packageScripts);
  const unsupportedCommands = commands
    .filter((command) => command.status === 'unsupported')
    .map(({ scriptName, command }) => ({ scriptName, command }));
  for (const command of unsupportedCommands) {
    diagnostics.push({ kind: 'lint-command-unsupported', ...command });
  }

  const degraded = diagnostics.length > 0;
  return {
    boundaries,
    evidence: {
      schemaVersion: 'lumin-lint-enforcement-evidence.v1',
      status: degraded ? 'degraded' : 'complete',
      configs,
      commands,
      unsupportedCommands,
      diagnostics,
    },
  };
}
