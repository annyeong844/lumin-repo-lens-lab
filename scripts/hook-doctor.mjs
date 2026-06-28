#!/usr/bin/env node

import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import {
  resolveAuditRoot,
  resolveWorkspaceRoot,
} from '../_lib/hook-path-safety.mjs';

const REPO_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const MANIFEST_REL = path.join('hooks', 'hooks.json');
const MANIFEST_PATH = path.join(REPO_ROOT, MANIFEST_REL);

function readHookManifest() {
  const parsed = JSON.parse(readFileSync(MANIFEST_PATH, 'utf8'));
  if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) {
    throw new Error('hooks manifest must be a JSON object');
  }
  if (!parsed.hooks || typeof parsed.hooks !== 'object' || Array.isArray(parsed.hooks)) {
    throw new Error('hooks manifest must contain an object-valued hooks field');
  }
  return parsed;
}

function activeHookEvents(manifest) {
  return Object.entries(manifest.hooks)
    .filter(([, groups]) => Array.isArray(groups) && groups.length > 0)
    .map(([event]) => event)
    .sort();
}

function main() {
  const workspaceRoot = resolveWorkspaceRoot(process.cwd()) ?? '<not-found>';
  const auditRoot = resolveAuditRoot(process.cwd()) ?? '<not-found>';
  const preimageStore =
    auditRoot === '<not-found>'
      ? '<not-found>'
      : existsSync(path.join(auditRoot, 'sessions', 'default-session', 'preimages'))
        ? 'present'
        : 'not-created';
  const eventStore =
    auditRoot === '<not-found>'
      ? '<not-found>'
      : existsSync(path.join(auditRoot, 'sessions', 'default-session', 'event-store'))
        ? 'present'
        : 'not-created';
  const manifest = readHookManifest();
  const activeEvents = activeHookEvents(manifest);

  console.log('hook doctor');
  console.log(`workspaceRoot: ${workspaceRoot}`);
  console.log(`auditRoot: ${auditRoot}`);
  console.log(`manifest: ${MANIFEST_REL}`);
  console.log(`activeHookEvents: ${activeEvents.length}`);
  console.log(`preimageStore: ${preimageStore}`);
  console.log(`eventStore: ${eventStore}`);
  if (activeEvents.length > 0) {
    console.log(`events: ${activeEvents.join(', ')}`);
  }
}

try {
  main();
} catch (error) {
  console.error(`hook doctor failed: ${error?.message ?? error}`);
  process.exit(1);
}
