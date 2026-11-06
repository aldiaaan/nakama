import { defineConfig } from 'oxfmt';

export const nakamaOxfmtConfig = defineConfig({
  printWidth: 120,
  sortImports: true,
  singleQuote: true,
  tabWidth: 4,
  sortTailwindcss: true,
  overrides: [
    {
      files: ['*.js', '*.ts', '*.jsx', '*.tsx', '*.mjs', '*.cjs'],
      options: {
        tabWidth: 2,
        jsxSingleQuote: false,
        trailingComma: 'es5',
        singleQuote: true,
      },
    },
    {
      files: ['*.css'],
      options: {
        singleQuote: false,
      },
    },
    {
      files: ['*.json'],
      options: {
        tabWidth: 4,
        trailingComma: 'none',
      },
    },
  ],
  ignorePatterns: [
    '**/build/*',
    '**/dist/*',
    '**/out/*',
    '**/infra/*',
    '**/vendor/*',
    '**/temp/*',
    '**/tmp/*',
    '**/.astro',
    '**/.next',
    '**/.tanstack',
    'infrastructure/*',
    'templates/*',
    'docsite/*',
    '.moon/*',
    '.temp/*',
  ],
});
