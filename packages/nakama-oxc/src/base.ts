import { defineConfig } from 'oxlint';

export const nakamaOxlintBaseConfig = defineConfig({
  plugins: ['typescript', 'import'],
  categories: {
    correctness: 'error',
    suspicious: 'warn',
  },
  rules: {
    'no-console': [
      'warn',
      {
        allow: ['assert', 'error', 'info', 'warn', 'debug', 'time', 'timeEnd', 'timeLog'],
      },
    ],
    'no-debugger': 'off',
    'no-param-reassign': 'warn',
    'default-param-last': 'error',
    'import/no-unassigned-import': [
      'warn',
      {
        allow: ['**/*.css'],
      },
    ],
    'no-else-return': 'error',
    'no-empty': 'warn',
    'no-unused-vars': [
      'warn',
      {
        ignoreRestSiblings: true,
      },
    ],
    'typescript/no-non-null-assertion': 'off',
    'typescript/no-explicit-any': 'off',
    'typescript/no-inferrable-types': 'error',
    'typescript/array-type': 'warn',
    'typescript/consistent-type-exports': 'warn',
    'typescript/consistent-type-imports': 'off',
  },
  overrides: [
    {
      files: ['**/*.astro'],
      rules: {
        'no-unused-vars': 'off',
      },
    },
  ],
});
