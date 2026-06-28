import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    exclude: ['node_modules/**', '.git/**', '.hex-skills/**', 'output/**', 'p6-corpus/**'],
    fileParallelism: false,
    include: ['tests/*.test.mjs'],
  },
});
