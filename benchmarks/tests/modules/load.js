console.log("Starting module loading benchmark...");

const modules = [
    'module1.js',
    'module2.js', 
    'module3.js',
    'module4.js',
    'module5.js'
];

const start = performance.now();

for (const module of modules) {
    console.log(`Loading ${module}...`);
    const loadedModule = require(`./${module}`);
    console.log(`${module} loaded:`, loadedModule.name);
}

const end = performance.now();
const duration = end - start;

console.log(`Module loading benchmark completed in ${duration.toFixed(2)}ms`);
console.log(`Total modules loaded: ${modules.length}`);