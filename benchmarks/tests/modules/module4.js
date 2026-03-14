const functions = {
    add: (a, b) => a + b,
    subtract: (a, b) => a - b,
    multiply: (a, b) => a * b,
    divide: (a, b) => a / b
};

const constants = {
    PI: Math.PI,
    E: Math.E,
    SQRT2: Math.SQRT2
};

module.exports = {
    name: 'Module 4',
    version: '1.0.0',
    functions,
    constants
};