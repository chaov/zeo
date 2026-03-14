declare module 'zeo' {
  export function init(engineType: number): number;
  export function execute(code: string): string;
  export function call(func: string, args: string): string;
  export function setGlobal(name: string, value: string): number;
  export function getGlobal(name: string): string;
  export function free(ptr: number): void;
  export function memoryUsage(): number;
  export function gc(): number;
}

export class ZeoRuntime {
  private initialized: boolean = false;

  constructor(engineType: number = 0) {
    const result = zeo.init(engineType);
    if (result === 0) {
      this.initialized = true;
    } else {
      throw new Error('Failed to initialize zeo');
    }
  }

  execute(code: string): any {
    if (!this.initialized) {
      throw new Error('zeo not initialized');
    }

    const result = zeo.execute(code);
    try {
      return JSON.parse(result);
    } catch {
      return result;
    }
  }

  call(func: string, args: any[]): any {
    if (!this.initialized) {
      throw new Error('zeo not initialized');
    }

    const argsStr = JSON.stringify(args);
    const result = zeo.call(func, argsStr);
    try {
      return JSON.parse(result);
    } catch {
      return result;
    }
  }

  setGlobal(name: string, value: any): void {
    if (!this.initialized) {
      throw new Error('zeo not initialized');
    }

    const valueStr = JSON.stringify(value);
    const result = zeo.setGlobal(name, valueStr);
    if (result !== 0) {
      throw new Error(`Failed to set global variable: ${name}`);
    }
  }

  getGlobal(name: string): any {
    if (!this.initialized) {
      throw new Error('zeo not initialized');
    }

    const result = zeo.getGlobal(name);
    try {
      return JSON.parse(result);
    } catch {
      return result;
    }
  }

  memoryUsage(): number {
    return zeo.memoryUsage();
  }

  gc(): void {
    const result = zeo.gc();
    if (result !== 0) {
      throw new Error('Failed to trigger garbage collection');
    }
  }

  evaluate(code: string): any {
    return this.execute(code);
  }
}

let globalRuntime: ZeoRuntime | null = null;

export function init(engineType: number = 0): ZeoRuntime {
  if (globalRuntime) {
    return globalRuntime;
  }

  globalRuntime = new ZeoRuntime(engineType);
  return globalRuntime;
}

export function getRuntime(): ZeoRuntime | null {
  return globalRuntime;
}

export default {
  init,
  getRuntime,
  ZeoRuntime,
};