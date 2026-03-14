console.log("Starting AI Agent execution benchmark...");

class SimpleAgent {
    constructor(name, config) {
        this.name = name;
        this.config = config;
        this.memory = [];
        this.context = {};
    }

    async think(input) {
        const startTime = performance.now();
        
        await this.processInput(input);
        await this.retrieveMemory();
        await this.generateResponse();
        await this.updateMemory();
        
        const endTime = performance.now();
        return {
            agent: this.name,
            processingTime: endTime - startTime,
            timestamp: Date.now()
        };
    }

    async processInput(input) {
        this.context.input = input;
        this.context.tokens = input.split(' ').length;
        this.context.entities = this.extractEntities(input);
    }

    extractEntities(text) {
        const entities = [];
        const words = text.split(/\s+/);
        
        words.forEach(word => {
            if (word.match(/^\d+$/)) {
                entities.push({ type: 'number', value: parseInt(word) });
            } else if (word.match(/^[A-Z]/)) {
                entities.push({ type: 'proper_noun', value: word });
            }
        });
        
        return entities;
    }

    async retrieveMemory() {
        this.context.relevantMemories = this.memory.filter(m => 
            m.timestamp > Date.now() - 3600000
        );
    }

    async generateResponse() {
        const response = {
            content: `Processed ${this.context.tokens} tokens`,
            confidence: Math.random() * 0.3 + 0.7,
            reasoning: this.generateReasoning()
        };
        this.context.response = response;
    }

    generateReasoning() {
        const steps = [
            'Analyzed input structure',
            'Identified key entities',
            'Retrieved relevant context',
            'Generated appropriate response',
            'Calculated confidence score'
        ];
        return steps;
    }

    async updateMemory() {
        this.memory.push({
            input: this.context.input,
            response: this.context.response,
            timestamp: Date.now()
        });
        
        if (this.memory.length > 100) {
            this.memory = this.memory.slice(-100);
        }
    }
}

class AgentOrchestrator {
    constructor() {
        this.agents = {};
        this.workflows = {};
    }

    registerAgent(name, config) {
        this.agents[name] = new SimpleAgent(name, config);
    }

    async executeWorkflow(workflowName, input) {
        const workflow = this.workflows[workflowName];
        if (!workflow) {
            throw new Error(`Workflow ${workflowName} not found`);
        }

        const results = [];
        let currentInput = input;

        for (const step of workflow.steps) {
            const agent = this.agents[step.agent];
            if (!agent) {
                throw new Error(`Agent ${step.agent} not found`);
            }

            const result = await agent.think(currentInput);
            results.push(result);
            currentInput = result;
        }

        return results;
    }
}

const orchestrator = new AgentOrchestrator();

orchestrator.registerAgent('analyzer', { role: 'analyzer', capabilities: ['text_analysis'] });
orchestrator.registerAgent('processor', { role: 'processor', capabilities: ['data_processing'] });
orchestrator.registerAgent('responder', { role: 'responder', capabilities: ['response_generation'] });

orchestrator.workflows['data_analysis'] = {
    steps: [
        { agent: 'analyzer', action: 'analyze' },
        { agent: 'processor', action: 'process' },
        { agent: 'responder', action: 'respond' }
    ]
};

async function runAgentBenchmark() {
    const testInputs = [
        "Analyze the sales data from Q1 2024",
        "Process customer feedback from last week",
        "Generate summary report for management",
        "Identify trends in user behavior data",
        "Calculate metrics for performance dashboard"
    ];

    const start = performance.now();

    console.log("Running individual agent tests...");
    for (const input of testInputs) {
        const agent = orchestrator.agents['analyzer'];
        const result = await agent.think(input);
        console.log(`Agent processed "${input.substring(0, 30)}..." in ${result.processingTime.toFixed(2)}ms`);
    }

    console.log("Running workflow tests...");
    for (const input of testInputs) {
        const results = await orchestrator.executeWorkflow('data_analysis', input);
        const totalTime = results.reduce((sum, r) => sum + r.processingTime, 0);
        console.log(`Workflow completed in ${totalTime.toFixed(2)}ms (${results.length} steps)`);
    }

    const end = performance.now();
    const duration = end - start;

    console.log(`AI Agent benchmark completed in ${duration.toFixed(2)}ms`);
    console.log("All agent operations completed successfully");
    
    return duration;
}

runAgentBenchmark().then(duration => {
    console.log(`Total benchmark duration: ${duration.toFixed(2)}ms`);
}).catch(error => {
    console.error('Benchmark failed:', error);
});