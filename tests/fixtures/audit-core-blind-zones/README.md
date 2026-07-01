# Audit-Core Blind-Zone Parity Fixtures

These fixtures are shared parity inputs for the JS-owned
`_lib/blind-zones.mjs` helper and the Rust `lumin-audit-core blind-zones-summary`
CLI.

Each case contains:

- `input`: the exact artifact payload shape passed to `detectBlindZones`.
- `expectedZones`: required `area`/`severity` pairs.
- `expectedDetails`: optional semantic assertions for selected product-visible
  fields on a zone. `path` is a list of object keys or array indexes starting
  at the zone root, and `equals` is the expected JSON value.
- `absentAreas`: zones that must not be emitted for that case.

The current Rust test suite consumes this file directly. When Node verification
is allowed, the JS parity runner should consume the same cases and compare JS
and Rust outputs from these inputs.
