export type CommandError = { type: 'DeviceInfo'; message: string } | { type: 'Internal'; message: string };

export class NakamaError extends Error {
  public type: string;

  constructor(commandError: CommandError | string) {
    if (typeof commandError === 'string') {
      super(commandError);
      this.type = 'Unknown';
    } else {
      super(commandError.message);
      this.type = commandError.type;
    }
    this.name = 'NakamaError';
    Object.setPrototypeOf(this, NakamaError.prototype);
  }
}
