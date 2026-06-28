Scan range: 42 TypeScript and JavaScript files. The strongest signal is a 4-file cycle in the pattern-matching helpers, so I would start there before touching smaller cleanup items.

The next step is one narrow slice: split the shared predicate helper out, rerun the audit, and make sure the cycle count drops while the public API stays unchanged.
