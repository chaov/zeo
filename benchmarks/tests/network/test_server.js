const http = require('http');

const PORT = 3000;

const server = http.createServer((req, res) => {
    const url = req.url;
    
    console.log(`Request received: ${url}`);
    
    // Simulate API endpoints
    if (url.startsWith('/api/')) {
        const endpoint = url.substring(5);
        
        setTimeout(() => {
            const responseData = {
                endpoint: endpoint,
                timestamp: Date.now(),
                data: {
                    message: `Response from ${endpoint}`,
                    randomValue: Math.random() * 1000,
                    array: Array(100).fill(0).map(() => Math.random())
                }
            };
            
            res.writeHead(200, { 'Content-Type': 'application/json' });
            res.end(JSON.stringify(responseData));
        }, Math.random() * 50); // Add some latency
    } else {
        res.writeHead(404, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify({ error: 'Not found' }));
    }
});

server.listen(PORT, () => {
    console.log(`Test HTTP server running on port ${PORT}`);
});

// Graceful shutdown
process.on('SIGTERM', () => {
    console.log('Shutting down test server...');
    server.close(() => {
        console.log('Server closed');
        process.exit(0);
    });
});