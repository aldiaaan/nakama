import { nakamaOxlintBaseConfig } from '@nakama/oxc/base';
import { nakamaOxlintReactConfig } from '@nakama/oxc/react';
import { defineConfig } from 'oxlint';

export default defineConfig({
  extends: [nakamaOxlintBaseConfig, nakamaOxlintReactConfig],
});
