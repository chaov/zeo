const fs = require('fs');
const path = require('path');

console.log("Starting file I/O benchmark...");

const testDir = './benchmark_test_files';
const testFile = path.join(testDir, 'test.txt');
const largeFile = path.join(testDir, 'large_file.bin');

if (!fs.existsSync(testDir)) {
    fs.mkdirSync(testDir, { recursive: true });
}

const start = performance.now();

console.log("1. Writing small file...");
const smallContent = "Hello, World! This is a test file for benchmarking.\n".repeat(100);
fs.writeFileSync(testFile, smallContent);
console.log("Small file written");

console.log("2. Reading small file...");
const readSmall = fs.readFileSync(testFile, 'utf8');
console.log(`Small file read: ${readSmall.length} characters`);

console.log("3. Writing large file (10MB)...");
const largeBuffer = Buffer.alloc(10 * 1024 * 1024);
for (let i = 0; i < largeBuffer.length; i++) {
    largeBuffer[i] = Math.floor(Math.random() * 256);
}
fs.writeFileSync(largeFile, largeBuffer);
console.log("Large file written");

console.log("4. Reading large file...");
const readLarge = fs.readFileSync(largeFile);
console.log(`Large file read: ${readLarge.length} bytes`);

console.log("5. File stats...");
const stats = fs.statSync(testFile);
console.log(`File stats: size=${stats.size}, modified=${stats.mtime}`);

console.log("6. Directory listing...");
const files = fs.readdirSync(testDir);
console.log(`Directory listing: ${files.length} files`);

console.log("7. Appending to file...");
fs.appendFileSync(testFile, "\nAppended content");
console.log("File appended");

console.log("8. Deleting files...");
fs.unlinkSync(testFile);
fs.unlinkSync(largeFile);
console.log("Files deleted");

const end = performance.now();
const duration = end - start;

console.log(`File I/O benchmark completed in ${duration.toFixed(2)}ms`);
console.log("All file operations completed successfully");