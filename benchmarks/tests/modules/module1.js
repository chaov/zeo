module.exports = {
    name: 'Module 1',
    version: '1.0.0',
    data: Array(1000).fill(0).map((_, i) => ({
        id: i,
        value: Math.random() * 1000
    }))
};