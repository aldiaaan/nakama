import { defineConfig } from 'oxlint';

export const nakamaOxlintReactConfig = defineConfig({
  plugins: ['react', 'jsx-a11y'],
  rules: {
    'jsx-a11y/anchor-has-content': 'off',
    'jsx-a11y/heading-has-content': 'off',
    'jsx-a11y/prefer-tag-over-role': 'off',
    'react/jsx-key': 'error',
    'react/no-array-index-key': 'warn',
    'react/no-danger': 'off',
    'react/no-children-prop': 'off',
    'react/jsx-no-useless-fragment': 'warn',
    'react/react-in-jsx-scope': 'off',
    'react/self-closing-comp': 'error',
  },
});
