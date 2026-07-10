# WSL Packaged Runtime Verification

> Date: 2026-07-10
> Status: verified repair

## Reported Failure

A real WSL dogfood run stopped with exit code 2 because `oxc-parser` could not
load its Linux native binding. The target repository was already dirty, so the
reproduction did not modify, clean, or reinstall anything in that repository.

## Isolated Reproduction

Two independent runtime conditions were found:

1. A non-login WSL shell selected Node 18.19.1 and npm 9.2.0. That toolchain is
   outside this package's `^20.19.0 || >=22.12.0` engine range. A clean
   production install added five packages, omitted the Linux OXC binding, and
   reproduced the native-binding error.
2. A login WSL shell selected Node 25.7.0 and npm 11.10.1. The same clean
   install added seven packages, installed the Linux glibc and musl OXC
   bindings, and parsed a fixture successfully.

The package lock already contains both Linux OXC binding packages. The checked
installation condition is therefore a WSL-local checkout and `node_modules`
tree created by a supported WSL Node/npm toolchain. A Windows dependency tree
under `/mnt/c` is not a portable Linux install.

## Packaged Audit-Core Defect

The checked-in Linux and Windows audit-core binaries both reported
`audit-core-js-runtime-bridge.v30`, but each reported only 46 of the 50 features
required by the v30 JS bridge. The missing features were:

- `symbolGraphEmbeddedRelativeMissingEvidence`
- `symbolGraphExternalDependencyInputFinalization`
- `symbolGraphExternalSourceUseAssemblyFinalization`
- `symbolGraphSfcGeneratedManifestExternalCountOnly`

With `LUMIN_AUDIT_CORE_NO_AUTO_BUILD=1`, WSL correctly rejected the stale Linux
binary. With auto-build enabled and Cargo available, the wrapper silently built
the packaged Rust source fallback, adding about 56 seconds to a tiny quick
audit. A Cargo-less installed package would stop instead.

The repair is to bump the bridge contract to v31 and rebuild every advertised
platform binary from the same source. Cargo remains a fallback and must not be
required for the advertised Linux package path.

An initial rebuild on the maintainer's Ubuntu 24.04 WSL host required
`GLIBC_2.39`, as did the previously checked-in Linux binary. That is too narrow
for a generic `linux-x64` package. The final binary was therefore rebuilt in a
Debian Bullseye Rust container and verified with `readelf` before packaging.

## Acceptance

- a supported WSL Node/npm install loads `oxc-parser` from a WSL-local package;
- Linux and Windows binaries report bridge contract v31 and the complete
  required feature set;
- a clean WSL quick audit completes with
  `LUMIN_AUDIT_CORE_NO_AUTO_BUILD=1` and Cargo absent from `PATH`;
- the run uses the packaged Linux binary and does not compile Rust sources;
- no files in the external dogfood repository are modified.

## Implementation Verification

The repaired skill was copied into a fresh WSL `/tmp` directory. A clean
production install under Node 25.7.0 and npm 11.10.1 installed seven packages,
and an OXC parse smoke confirmed that the Linux native binding loaded.

The audit process then ran with Cargo removed from `PATH`,
`LUMIN_AUDIT_CORE_NO_AUTO_BUILD=1`, and
`LUMIN_REPO_LENS_NO_AUTO_INSTALL=1`. A quick audit over a two-file fixture
completed without compiling Rust sources and wrote:

| Measurement | Value |
|---|---:|
| produced artifacts | 16 |
| manifest profile | `quick` |
| reported blind zones | 0 |
| Cargo visible during audit | no |
| audit-core auto-build allowed | no |
| packaged Linux maximum GLIBC requirement | 2.30 |

Both packaged binaries report `audit-core-js-runtime-bridge.v31` with all 50
required features. The Windows and Linux packaged wrappers also pass the full
synthetic result-file contract probe. That probe exposed one stale fixture: it
supplied one SFC script-src use both as a legacy count and as a Rust assembly
record. The fixture now matches the production request boundary by sending a
zero legacy count and allowing Rust assembly to produce the single use.

The package builder now performs non-executing validation for binaries supplied
for another platform. It checks binary format and architecture, embedded v31
and required-feature markers, and the Linux GLIBC floor. Verification proved
both hard-stop branches: the previous v30 Linux binary was rejected for the
missing v31 marker, and a v31 host build requiring GLIBC 2.39 was rejected for
exceeding the 2.31 baseline. The final GLIBC 2.30 binary also executed its
runtime contract successfully inside a Debian Bullseye container.
