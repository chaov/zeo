import type { AgentConfig, AgentResponse, Message } from '../core/types';
import { Agent } from '../core/agent';
import { init } from '../zeo/binding';
import { FileTool, SystemTool, HttpTool } from '../tools/builtin';

export class ChatAgent extends Agent {
  constructor(config: AgentConfig) {
    super(config);
  }

  async execute(message: Message): Promise<AgentResponse> {
    const session = await this.sessionManager.getOrCreate(message.sessionId || 'default');
    const context = await this.buildContext(session, message);

    const response = await this.llmClient.chat({
      messages: context,
      tools: this.toolRegistry.getToolSchemas(),
    });

    if (response.toolCalls && response.toolCalls.length > 0) {
      const toolResults = [];

      for (const toolCall of response.toolCalls) {
        const result = await this.onToolCall(toolCall);
        toolResults.push(result);

        if (result.error) {
          response.content += `\nTool error: ${result.error}`;
        } else {
          response.content += `\nTool result: ${JSON.stringify(result.result)}`;
        }
      }

      this.sessionManager.addMessage(session.id, {
        role: 'assistant',
        content: response.content,
        toolCalls: response.toolCalls,
        toolResults,
      });
    } else {
      this.sessionManager.addMessage(session.id, {
        role: 'assistant',
        content: response.content,
      });
    }

    this.sessionManager.addMessage(session.id, message);

    return response;
  }
}

export class TaskAgent extends Agent {
  constructor(config: AgentConfig) {
    super(config);
  }

  async execute(message: Message): Promise<AgentResponse> {
    const session = await this.sessionManager.getOrCreate(message.sessionId || 'default');
    
    const taskPrompt = `
Task: ${message.content}

Please analyze this task and determine what actions need to be taken.
Break down the task into steps and use available tools to complete each step.
`;

    const taskMessage: Message = {
      ...message,
      content: taskPrompt,
    };

    const context = await this.buildContext(session, taskMessage);

    const response = await this.llmClient.chat({
      messages: context,
      tools: this.toolRegistry.getToolSchemas(),
    });

    if (response.toolCalls && response.toolCalls.length > 0) {
      for (const toolCall of response.toolCalls) {
        const result = await this.onToolCall(toolCall);
        
        if (result.error) {
          response.content += `\nTool error: ${result.error}`;
        } else {
          response.content += `\nTool result: ${JSON.stringify(result.result)}`;
        }
      }
    }

    this.sessionManager.addMessage(session.id, message);
    this.sessionManager.addMessage(session.id, {
      role: 'assistant',
      content: response.content,
      toolCalls: response.toolCalls,
    });

    return response;
  }
}

export function createChatAgent(config: Partial<AgentConfig> = {}): ChatAgent {
  const defaultConfig: AgentConfig = {
    name: 'chat-agent',
    systemPrompt: 'You are a helpful AI assistant.',
    maxHistory: 10,
    temperature: 0.7,
    maxTokens: 2048,
    llm: {
      provider: 'openai',
      model: 'gpt-4',
      streaming: true,
    },
    ...config,
  };

  const agent = new ChatAgent(defaultConfig);
  
  init(0);
  agent.getToolRegistry().register(new FileTool());
  agent.getToolRegistry().register(new SystemTool());
  agent.getToolRegistry().register(new HttpTool());

  return agent;
}

export function createTaskAgent(config: Partial<AgentConfig> = {}): TaskAgent {
  const defaultConfig: AgentConfig = {
    name: 'task-agent',
    systemPrompt: 'You are a task execution AI. Break down tasks and use tools to complete them.',
    maxHistory: 20,
    temperature: 0.3,
    maxTokens: 4096,
    llm: {
      provider: 'openai',
      model: 'gpt-4',
      streaming: false,
    },
    ...config,
  };

  const agent = new TaskAgent(defaultConfig);
  
  init(0);
  agent.getToolRegistry().register(new FileTool());
  agent.getToolRegistry().register(new SystemTool());
  agent.getToolRegistry().register(new HttpTool());

  return agent;
}