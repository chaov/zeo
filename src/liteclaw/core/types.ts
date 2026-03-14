export interface Message {
  role: 'system' | 'user' | 'assistant' | 'tool';
  content: string;
  toolCalls?: ToolCall[];
  toolResults?: ToolResult[];
  metadata?: Record<string, any>;
}

export interface ToolCall {
  id: string;
  name: string;
  parameters: any;
}

export interface ToolResult {
  id: string;
  result: any;
  error?: string;
}

export interface AgentConfig {
  name: string;
  systemPrompt?: string;
  maxHistory?: number;
  temperature?: number;
  maxTokens?: number;
  llm: LLMConfig;
}

export interface LLMConfig {
  provider: string;
  model: string;
  apiKey?: string;
  baseUrl?: string;
  streaming?: boolean;
}

export interface AgentResponse {
  content: string;
  toolCalls?: ToolCall[];
  metadata?: Record<string, any>;
}

export interface Session {
  id: string;
  createdAt: Date;
  updatedAt: Date;
  messages: Message[];
  metadata: Record<string, any>;
}

export interface ChannelConfig {
  name: string;
  type: string;
  config: Record<string, any>;
}