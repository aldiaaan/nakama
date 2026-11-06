import { nakamaOxlintBaseConfig } from '@nakama/oxc/base';
import { nakamaOxlintReactConfig } from '@nakama/oxc/react';
import { defineConfig } from 'oxlint';

export default defineConfig({
  extends: [nakamaOxlintBaseConfig, nakamaOxlintReactConfig],
  ignorePatterns: [
    'src/components/ui/**',
    'src/lib/highlighter.tsx',
    'src/hooks/use-highlighted-code.ts',
    'src/hooks/use-mobile.ts',
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
