import type { Message, AgentConfig, AgentResponse, Session, ToolCall, ToolResult } from './types';
import { ToolRegistry } from '../tools/registry';
import { SessionManager } from './session';
import { LLMClient } from '../llm/client';

export abstract class Agent {
  protected toolRegistry: ToolRegistry;
  protected sessionManager: SessionManager;
  protected llmClient: LLMClient;
  protected config: AgentConfig;

  constructor(config: AgentConfig) {
    this.config = config;
    this.toolRegistry = new ToolRegistry();
    this.sessionManager = new SessionManager();
    this.llmClient = new LLMClient(config.llm);
  }

  abstract execute(message: Message): Promise<AgentResponse>;

  async onMessage(message: Message): Promise<void> {
    const response = await this.execute(message);
    return response;
  }

  async onToolCall(toolCall: ToolCall): Promise<ToolResult> {
    try {
      const result = await this.toolRegistry.execute(
        toolCall.name,
        toolCall.parameters,
        {
          agentId: this.config.name,
          sessionId: 'default',
        }
      );

      return {
        id: toolCall.id,
        result,
      };
    } catch (error) {
      return {
        id: toolCall.id,
        result: null,
        error: error instanceof Error ? error.message : String(error),
      };
    }
  }

  protected async buildContext(session: Session, message: Message): Promise<Message[]> {
    const systemPrompt = this.config.systemPrompt || 'You are a helpful AI assistant.';
    const maxHistory = this.config.maxHistory || 10;

    const context: Message[] = [
      { role: 'system', content: systemPrompt },
      ...session.messages.slice(-maxHistory),
      message,
    ];

    return context;
  }

  async start(): Promise<void> {
    await this.sessionManager.initialize();
  }

  async stop(): Promise<void> {
    await this.sessionManager.cleanup();
  }

  getToolRegistry(): ToolRegistry {
    return this.toolRegistry;
  }

  getSessionManager(): SessionManager {
    return this.sessionManager;
  }
}