import { invoke, type InvokeArgs } from '@tauri-apps/api/core';

import { type CommandError, NakamaError } from './errors';
import { DaemonResource } from './resources/daemon';
import { DeviceResource } from './resources/device';
import { ExampleDaemonResource } from './resources/example-daemon';
import { NotificationResource } from './resources/notification';
import { ProfilersResource } from './resources/profilers';
import { SettingsResource } from './resources/settings';

export class NakamaApi {
  readonly daemon = new DaemonResource(this);
  readonly device = new DeviceResource(this);
  readonly exampleDaemon = new ExampleDaemonResource(this);
  readonly notification = new NotificationResource(this);
  readonly profilers = new ProfilersResource(this);
  readonly settings = new SettingsResource(this);

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
