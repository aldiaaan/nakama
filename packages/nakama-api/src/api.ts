import { invoke, type InvokeArgs } from '@tauri-apps/api/core';

import { type CommandError, NakamaError } from './errors';
import { DeviceResource } from './resources/device';

export class NakamaApi {
  readonly device = new DeviceResource(this);

  invoke<T>(command: string, args?: InvokeArgs): Promise<T> {
    return this.safeInvoke<T>(command, args);
  }

  private async safeInvoke<T>(command: string, args?: InvokeArgs): Promise<T> {
    try {
      return await invoke<T>(command, args);
    } catch (rawError) {
      throw new NakamaError(rawError as CommandError | string);
    }
  }
}
