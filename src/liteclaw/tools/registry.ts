import type { Tool, ToolHandler, ToolContext, Permission } from './types';

export class ToolRegistry {
  private tools: Map<string, Tool> = new Map();
  private permissions: Map<string, Permission[]> = new Map();

  register(tool: Tool): void {
    this.tools.set.set(tool.name, tool);
    if (tool.permissions) {
      this.permissions.set(tool.name, tool.permissions);
    }
  }

  unregister(name: string): boolean {
    const removed = this.tools.delete(name);
    this.permissions.delete(name);
    return removed;
  }

  get(name: string): Tool | undefined {
    return this.tools.get(name);
  }

  async execute(
    name: string,
    parameters: any,
    context: ToolContext
  ): Promise<any> {
    const tool = this.get(name);
    if (!tool) {
      throw new Error(`Tool not found: ${name}`);
    }

    await this.checkPermissions(name, context);

    return await tool.handler(parameters, context);
  }

  list(): Tool[] {
    return Array.from(this.tools.values());
  }

  getToolSchemas(): ToolSchema[] {
    return this.list().map(tool => ({
      name: tool.name,
      description: tool.description,
      parameters: tool.parameters,
    }));
  }

  private async checkPermissions(name: string, context: ToolContext): Promise<void> {
    const permissions = this.permissions.get(name);
    if (!permissions || permissions.length === 0) return;

    for (const permission of permissions) {
      await context.permissionManager?.check(permission);
    }
  }
}

export interface ToolSchema {
  name: string;
  description: string;
  parameters: any;
}