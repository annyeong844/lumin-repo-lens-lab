// ESLint flat config — v1.8.2.
//
// Philosophy: catch new smell before it accumulates. Silent catches were
// the original 1.4.0-era concession (`allowEmptyCatch: true` with 25
// existing sites on the debt list). All 25 were audited and either
// replaced with `fileExists()` / `dirExists()` / `pathExists()` helpers
// (resolver-core / alias-map) or annotated with an explanatory comment.
// The rule is now strict — a genuinely intentional empty catch must at
// minimum carry a comment explaining the swallowed error.

export default [
  {
    ignores: [
      'node_modules/**',
      'audit-artifacts/**',
      'audit-artifacts-smoke/**',
      'output/**',
      'p6-corpus/**',
      'tests/fixture-*/**',
    ],
  },
  {
    files: ['**/*.mjs'],
    languageOptions: {
      ecmaVersion: 2024,
      sourceType: 'module',
      globals: {
        // Node globals we rely on
        process: 'readonly',
        console: 'readonly',
        Buffer: 'readonly',
        URL: 'readonly',
        URLSearchParams: 'readonly',
        setTimeout: 'readonly',
        clearTimeout: 'readonly',
        setImmediate: 'readonly',
        clearImmediate: 'readonly',
        __dirname: 'readonly',
        __filename: 'readonly',
      },
    },
    rules: {
      // Correctness — errors that catch real bugs
      'no-undef': 'error',
      'no-unused-vars': ['warn', {
        argsIgnorePattern: '^_',
        varsIgnorePattern: '^_',
        caughtErrors: 'none', // don't nag on `catch (e)` where e unused
      }],
      'no-unreachable': 'error',
      'no-const-assign': 'error',
      'no-dupe-keys': 'error',
      'no-duplicate-case': 'error',
      'no-empty': ['error', { allowEmptyCatch: false }],
      'no-func-assign': 'error',
      'no-invalid-regexp': 'error',
      'no-irregular-whitespace': 'error',
      'no-sparse-arrays': 'warn',
      'no-unexpected-multiline': 'error',
      'use-isnan': 'error',
      'valid-typeof': 'error',

      // Style — warnings only, don't block CI
      'prefer-const': 'warn',
      'no-var': 'error',
      'eqeqeq': ['warn', 'always', { null: 'ignore' }],
    },
  },
  {
    // Engine modules may be used by root CLIs, but must not reach back into
    // root scripts. Keep orchestration above `_lib/`, not hidden inside it.
    files: ['_lib/*.mjs'],
    rules: {
      'no-restricted-imports': ['error', {
        patterns: [
          {
            group: ['../*.mjs'],
            message: 'Root scripts are public orchestration entrypoints; move shared behavior into _lib instead.',
          },
        ],
      }],
    },
  },
  {
    // Tests get a looser regime — fixture construction often has unused
    // bindings and intentionally odd patterns.
    files: ['tests/**/*.mjs'],
    rules: {
      'no-unused-vars': 'off',
    },
  },
];
