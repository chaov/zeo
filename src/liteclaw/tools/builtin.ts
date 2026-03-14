import type { Tool, ToolHandler, ToolContext } from './types';
import * as fs from 'fs/promises';

export class FileTool implements Tool {
  name = 'file';
  description = 'File operations (read, write, delete)';
  parameters = {
    type: 'object',
    properties: {
      action: {
        type: 'string',
        enum: ['read', 'write', 'delete', 'exists'],
        description: 'The action to perform',
      },
      path: {
        type: 'string',
        description: 'The file path',
      },
      content: {
        type: 'string',
        description: 'Content to write (for write action)',
      },
    },
    required: ['action', 'path'],
  };

  async handler(params: any, context: ToolContext): Promise<any> {
    const { action, path, content } = params;

    switch (action) {
      case 'read':
        return await fs.readFile(path, 'utf-8');
      case 'write':
        await fs.writeFile(path, content, 'utf-8');
        return { success: true };
      case 'delete':
        await fs.unlink(path);
        return { success: true };
      case 'exists':
        try {
          await fs.access(path);
          return { exists: true };
        } catch {
          return { exists: false };
        }
      default:
        throw new Error(`Unknown action: ${action}`);
    }
  }
}

export class SystemTool implements Tool {
  name = 'system';
  description = 'System operations';
  parameters = {
    type: 'object',
    properties: {
      action: {
        type: 'string',
        enum: ['info', 'env', 'cwd'],
        description: 'The action to perform',
      },
      key: {
        type: 'string',
        description: 'Environment variable key (for env action)',
      },
    },
    required: ['action'],
  };

  async handler(params: any, context: ToolContext): Promise<any> {
    const { action, key } = params;

    switch (action) {
      case 'info':
        return {
          platform: process.platform,
          arch: process.arch,
          nodeVersion: process.version,
          pid: process.pid,
        };
      case 'env':
        if (key) {
          return process.env[key];
        }
        return process.env;
      case 'cwd':
        return process.cwd();
      default:
        throw new Error(`Unknown action: ${action}`);
    }
  }
}

export class HttpTool implements Tool {
  name = 'http';
  description = 'HTTP requests';
  parameters = {
    type: 'object',
    properties: {
      url: {
        type: 'string',
        description: 'The URL to request',
      },
      method: {
        type: 'string',
        enum: ['GET', 'POST', 'PUT', 'DELETE'],
        description: 'HTTP method',
      },
      headers: {
        type: 'object',
        description: 'Request headers',
      },
      body: {
        type: 'string',
        description: 'Request body',
      },
    },
    required: ['url'],
  };

  async handler(params: any, context: ToolContext): Promise<any> {
    const { url, method = 'GET', headers = {}, body } = params;

    try {
      const response = await fetch(url, {
        method,
        headers,
        body: body ? JSON.stringify(body) : undefined,
      });

      const text = await response.text();
      let data;
      try {
        data = JSON.parse(text);
      } catch {
        data = text;
      }

      return {
        status: response.status,
        statusText: response.statusText,
        headers: Object.fromEntries(response.headers.entries()),
        data,
      };
    } catch (error) {
      throw new Error(`HTTP request failed: ${error}`);
    }
  }
}

export function createDefaultTools(): Tool[] {
  return [
    new File(),
    new SystemTool(),
    new HttpTool(),
  ];
}