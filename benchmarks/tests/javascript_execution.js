console.log("Starting JavaScript execution benchmark...");

function fibonacci(n) {
    if (n <= 1) return n;
    return fibonacci(n - 1) + fibonacci(n - 2);
}

function matrixMultiplication(size) {
    const A = Array(size).fill(0).map(() => Array(size).fill(0).map(() => Math.random()));
    const B = Array(size).fill(0).map(() => Array(size).fill(0).map(() => Math.random()));
    const C = Array(size).fill(0).map(() => Array(size).fill(0));

    for (let i = 0; i < size; i++) {
        for (let j = 0; j < size; j++) {
            for (let k = 0; k k++) {
                C[i][j] += A[i][k] * B[k][j];
            }
        }
    }
    return C;
}

function stringOperations(iterations) {
    let result = "";
    for (let i = 0; i < iterations; i++) {
        result += "test string " + i + " ";
        result = result.toUpperCase();
        result = result.toLowerCase();
        result = result.trim();
    }
    return result.length;
}

function arrayOperations(size) {
    const arr = Array(size).fill(0).map((_, i) => i);
    
    const filtered = arr.filter(x => x % 2 === 0);
    const mapped = filtered.map(x => x * 2);
    const reduced = mapped.reduce((acc, x) => acc + x, 0);
    
    return reduced;
}

function objectOperations(count) {
    const objects = [];
    for (let i = 0; i < count; i++) {
        objects.push({
            id: i,
            name: `object_${i}`,
            value: Math.random() * 1000,
            metadata: {
                created: Date.now(),
                tags: ['tag1', 'tag2', 'tag3']
            }
        });
    }
    
    const sum = objects.reduce((acc, obj) => acc + obj.value, 0);
    return sum;
}

const start = performance.now();

console.log("Running Fibonacci(35)...");
const fibResult = fibonacci(35);
console.log(`Fibonacci result: ${fibResult}`);

console.log("Running matrix multiplication (50x50)...");
const matrixResult = matrixMultiplication(50);
console.log(`Matrix multiplication completed`);

console.log("Running string operations...");
const stringResult = stringOperations(10000);
console.log(`String operations result: ${stringResult}`);

console.log("Running array operations...");
const arrayResult = arrayOperations(100000);
console.log(`Array operations result: ${arrayResult}`);

console.log("Running object operations...");
const objectResult = objectOperations(10000);
console.log(`Object operations result: ${objectResult}`);

const end = performance.now();
const duration = end - start;

console.log(`JavaScript execution benchmark completed in ${duration.toFixed(2)}ms`);
console.log(`All operations completed successfully`);