#!/bin/bash

# Continuous Performance Monitoring Script
# This script runs benchmarks periodically and tracks performance over time

set -e

RESULTS_DIR="./reports/trends"
mkdir -p $RESULTS_DIR

INTERVAL=${1:-3600}  # Default: every hour
MAX_RESULTS=${2:-168}  # Default: keep 7 days of hourly results

echo "Starting continuous performance monitoring..."
echo "Interval: $INTERVAL seconds"
echo "Max results to keep: $MAX_RESULTS"
echo ""

while true; do
    TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
    RESULT_FILE="$RESULTS_DIR/benchmark_$TIMESTAMP.json"
    
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] Running benchmark..."
    
    # Run benchmark
    cargo run --release --quiet
    
    # Move results to timestamped file
    if [ -f "./reports/benchmark_report.json" ]; then
        cp "./reports/benchmark_report.json" "$RESULT_FILE"
        echo "[$(date '+%Y-%m-%d %H:%M:%S')] Results saved to $RESULT_FILE"
    fi
    
    # Clean up old results
    RESULT_COUNT=$(ls -1 $RESULTS_DIR/*.json 2>/dev/null | wc -l)
    if [ $RESULT_COUNT -gt $MAX_RESULTS ]; then
        echo "[$(date '+%Y-%m-%d %H:%M:%S')] Cleaning up old results..."
        ls -1t $RESULTS_DIR/*.json | tail -n +$((MAX_RESULTS + 1)) | xargs rm -f
    fi
    
    # Generate trend report
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] Generating trend report..."
    ./scripts/generate_trend_report.sh
    
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] Next run in $INTERVAL seconds..."
    echo "--------------------------------------------------"
    
    sleep $INTERVAL
done