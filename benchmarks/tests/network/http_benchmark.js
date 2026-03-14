console.log("Starting network request benchmark...");

const http = require('http');

const testUrls = [
    'http://localhost:3000/api/test1',
    'http://localhost:3000/api/test2',
    'http://localhost:3000/api/test3'
];

async function makeRequest(url) {
    return new Promise((resolve, reject) => {
        const startTime = performance.now();
        
        http.get(url, (res) => {
            let data = '';
            
            res.on('data', (chunk) => {
                data += chunk;
            });
            
            res.on('end', () => {
                const endTime = performance.now();
                resolve({
                    url,
                    status: res.statusCode,
                    dataSize: data.length,
                    duration: endTime - startTime
                });
            });
        }).on('error', (error) => {
            reject(error);
        });
    });
}

async function runConcurrentRequests(urls, concurrency = 3) {
    const results = [];
    
    for (let i = 0; i < urls.length; i += concurrency) {
        const batch = urls.slice(i, i + concurrency);
        const batchResults = await Promise.all(
            batch.map(url => makeRequest(url).catch(err => ({ url, error: err.message })))
        );
        results.push(...batchResults);
    }
    
    return results;
}

async function runSequentialRequests(urls) {
    const results = [];
    
    for (const url of urls) {
        try {
            const result = await makeRequest(url);
            results.push(result);
        } catch (error) {
            results.push({ url, error: error.message });
        }
    }
    
    return results;
}

const start = performance.now();

console.log("Running sequential requests...");
const sequentialResults = await runSequentialRequests(testUrls);
console.log(`Sequential requests completed: ${sequentialResults.length} requests`);

console.log("Running concurrent requests...");
const concurrentResults = await runConcurrentRequests(testUrls);
console.log(`Concurrent requests completed: ${concurrentResults.length} requests`);

const end = performance.now();
const duration = end - start;

console.log(`Network benchmark completed in ${duration.toFixed(2)}ms`);

sequentialResults.forEach((result, index) => {
    if (result.error) {
        console.log(`Request ${index + 1} failed: ${result.error}`);
    } else {
        console.log(`Request ${index + 1}: ${result.duration.toFixed(2)}ms, Status: ${result.status}`);
    }
});

console.log("Network benchmark completed successfully");