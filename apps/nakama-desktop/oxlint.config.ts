import { nakamaOxlintBaseConfig } from '@nakama/oxc/base';
import { nakamaOxlintReactConfig } from '@nakama/oxc/react';
import { defineConfig } from 'oxlint';

export default defineConfig({
  extends: [nakamaOxlintBaseConfig, nakamaOxlintReactConfig],
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
    'src-tauri/target/**',
  ],
});
