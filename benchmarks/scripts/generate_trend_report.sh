#!/bin/bash

# Generate Performance Trend Report
# This script analyzes historical benchmark data and generates trend reports

RESULTS_DIR="./reports/trends"
OUTPUT_DIR="./reports"
TREND_FILE="$OUTPUT_DIR/performance_trends.md"

echo "Generating performance trend report..."

# Check if we have results
if [ ! -d "$RESULTS_DIR" ] || [ -z "$(ls -A $RESULTS_DIR)" ]; then
    echo "No historical results found in $RESULTS_DIR"
    exit 1
fi

# Create markdown report
cat > "$TREND_FILE" << 'EOF'
# Zeo Performance Trends

This report shows performance trends over time based on historical benchmark data.

## Summary

EOF

# Get all result files sorted by date
RESULT_FILES=$(ls -1t $RESULTS_DIR/*.json)

# Count total runs
TOTAL_RUNS=$(echo "$RESULT_FILES" | wc -l)
echo "Total benchmark runs: $TOTAL_RUNS" >> "$TREND_FILE"
echo "" >> "$TREND_FILE"

# Extract and analyze data for each scenario
echo "## Performance Trends by Scenario" >> "$TREND_FILE"
echo "" >> "$TREND_FILE"

SCENARIOS=("startup_time" "javascript_execution" "module_loading" "file_io" "network_requests" "ai_agent_execution")

for scenario in "${SCENARIOS[@]}"; do
    echo "### $scenario" >> "$TREND_FILE"
    echo "" >> "$TREND_FILE"
    
    # Extract values for this scenario across all runs
    VALUES=$(for file in $RESULT_FILES; do
        if [ -f "$file" ]; then
            # Extract Zeo value for this scenario
            grep -A 5 "\"scenario\": \"$scenario\"" "$file" | grep "\"target\": \"zeo\"" -A 2 | grep "\"value\"" | head -1 | sed 's/.*"value": \([0-9.]*\).*/\1/'
        fi
    done)
    
    # Calculate statistics
    VALUE_COUNT=$(echo "$VALUES" | grep -c "[0-9]" || echo "0")
    
    if [ $VALUE_COUNT -gt 0 ]; then
        AVG=$(echo "$VALUES" | awk '{sum+=$1} END {print sum/NR}')
        MIN=$(echo "$VALUES" | sort -n | head -1)
        MAX=$(echo "$VALUES" | sort -n | tail -1)
        
        echo "- **Average:** ${AVG} ms" >> "$TREND_FILE"
        echo "- **Minimum:** ${MIN} ms" >> "$TREND_FILE"
        echo "- **Maximum:** ${MAX} ms" >> "$TREND_FILE"
        echo "- **Data points:** $VALUE_COUNT" >> "$TREND_FILE"
    else
        echo "No data available" >> "$TREND_FILE"
    fi
    
    echo "" >> "$TREND_FILE"
done

# Generate simple ASCII chart
echo "## Performance Charts" >> "$TREND_FILE"
echo "" >> "$TREND_FILE"

for scenario in "${SCENARIOS[@]}"; do
    echo "### $scenario Trend" >> "$TREND_FILE"
    echo "" >> "$TREND_FILE"
    
    VALUES=$(for file in $RESULT_FILES; do
        if [ -f "$file" ]; then
            grep -A 5 "\"scenario\": \"$scenario\"" "$file" | grep "\"target\": \"zeo\"" -A 2 | grep "\"value\"" | head -1 | sed 's/.*"value": \([0-9.]*\).*/\1/'
        fi
    done)
    
    # Create simple chart
    echo '```' >> "$TREND_FILE"
    echo "$VALUES" | nl -w 3 -s ': ' | while read line; do
        VALUE=$(echo "$line" | awk '{print $2}')
        INDEX=$(echo "$line" | awk '{print $1}')
        
        # Scale value for chart (normalize to 50 chars max)
        CHART_WIDTH=$(echo "scale=0; $VALUE / 10" | bc 2>/dev/null || echo "1")
        CHART_WIDTH=${CHART_WIDTH#.*}
        if [ $CHART_WIDTH -gt 50 ]; then
            CHART_WIDTH=50
        fi
        
        BAR=$(printf "%${CHART_WIDTH}s" | tr ' ' '=')
        printf "%3d: %s %.2f\n" "$INDEX" "$BAR" "$VALUE" >> "$TREND_FILE"
    done
    echo '```' >> "$TREND_FILE"
    echo "" >> "$TREND_FILE"
done

echo "Trend report generated: $TREND_FILE"