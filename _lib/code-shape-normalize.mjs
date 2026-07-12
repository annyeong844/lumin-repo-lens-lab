// Parser-free code-shape normalization shared by extraction and post-write.

// Collapse whitespace runs outside string, template, and comment ranges while
// preserving literal contents. This is intentionally a small state machine,
// not a parser; importing it must never load a native parser binding.
export function normalizeCodeShape(raw) {
  if (!raw) return '';
  const out = [];
  let state = 'code';
  let prevSpace = false;
  let i = 0;
  while (i < raw.length) {
    const c = raw[i];
    const next = raw[i + 1];

    if (state === 'single') {
      out.push(c);
      if (c === '\\' && i + 1 < raw.length) { out.push(next); i += 2; continue; }
      if (c === "'") state = 'code';
      i++;
      continue;
    }
    if (state === 'double') {
      out.push(c);
      if (c === '\\' && i + 1 < raw.length) { out.push(next); i += 2; continue; }
      if (c === '"') state = 'code';
      i++;
      continue;
    }
    if (state === 'template') {
      out.push(c);
      if (c === '\\' && i + 1 < raw.length) { out.push(next); i += 2; continue; }
      if (c === '`') state = 'code';
      i++;
      continue;
    }
    if (state === 'line-comment') {
      out.push(c);
      if (c === '\n') state = 'code';
      i++;
      continue;
    }
    if (state === 'block-comment') {
      out.push(c);
      if (c === '*' && next === '/') { out.push(next); state = 'code'; i += 2; continue; }
      i++;
      continue;
    }

    if (c === "'") { state = 'single'; out.push(c); prevSpace = false; i++; continue; }
    if (c === '"') { state = 'double'; out.push(c); prevSpace = false; i++; continue; }
    if (c === '`') { state = 'template'; out.push(c); prevSpace = false; i++; continue; }
    if (c === '/' && next === '/') { state = 'line-comment'; out.push(c, next); prevSpace = false; i += 2; continue; }
    if (c === '/' && next === '*') { state = 'block-comment'; out.push(c, next); prevSpace = false; i += 2; continue; }

    if (c === ' ' || c === '\t' || c === '\n' || c === '\r') {
      if (!prevSpace) {
        out.push(' ');
        prevSpace = true;
      }
      i++;
      continue;
    }

    out.push(c);
    prevSpace = false;
    i++;
  }

  let normalized = out.join('').trim();
  if (normalized.endsWith(';')) normalized = normalized.slice(0, -1).trimEnd();
  return normalized;
}
