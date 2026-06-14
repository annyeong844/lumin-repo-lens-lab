// Used by index.ts — alive
export function formatDate(d: Date): string {
  return d.toISOString();
}

// PLANTED: dead — no consumer anywhere
export function formatTimestamp(d: Date): string {
  return d.getTime().toString();
}

// PLANTED: dead — exports a wrapper of an internal helper, but nothing imports it
export function formatTime(d: Date): string {
  return helper(d);
}

// Internal — not exported, used by formatTime
function helper(d: Date): string {
  return d.toTimeString();
}
