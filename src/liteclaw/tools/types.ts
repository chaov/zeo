export interface Tool {
  name: string;
  description: string;
  parameters: any;
  handler: ToolHandler;
  permissions?: Permission[];
}

export type ToolHandler = (params: any, context: ToolContext) => Promise<any>;

export interface ToolContext {
  agentId: string;
  sessionId: string;
  permissionManager?: PermissionManager;
  metadata?: Record<string, any>;
}

export interface PermissionManager {
  check(permission: Permission): Promise<void>;
}

export type Permission =
  | { type: 'file'; action: 'read' | 'write' | 'delete'; path: string }
  | { type: 'network'; action: 'http' | 'websocket'; url?: string }
  | { type: 'system'; action: 'execute'; command?: string }
  | { type: 'database'; action: 'read' | 'write'; table?: string };