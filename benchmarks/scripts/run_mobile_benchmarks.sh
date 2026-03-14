#!/bin/bash

# Mobile Benchmark Runner for iOS and Android
# This script runs performance tests on mobile devices

set -e

DEVICE_TYPE=""
PLATFORM=""

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --ios)
            PLATFORM="ios"
            shift
            ;;
        --android)
            PLATFORM="android"
            shift
            ;;
        --device)
            DEVICE_TYPE="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

if [ -z "$PLATFORM" ]; then
    echo "Usage: $0 --ios|--android [--device DEVICE_ID]"
    exit 1
fi

echo "Running mobile benchmarks on $PLATFORM..."

if [ "$PLATFORM" == "ios" ]; then
    if ! command -v xcrun &> /dev/null; then
        echo "Error: xcrun not found. This script requires Xcode tools."
        exit 1
    fi
    
    if [ -n "$DEVICE_TYPE" ]; then
        DEVICE_ID=$DEVICE_TYPE
    else
        DEVICE_ID=$(xcrun xctrace list devices | grep "iPhone" | head -1 | grep -oE '[A-F0-9]{8}-[A-F0-9]{4}-[A-F0-9]{4}-[A-F0-9]{4}-[A-F0-9]{12}')
    fi
    
    echo "Using iOS device: $DEVICE_ID"
    
    # Deploy and run benchmark app
    echo "Deploying benchmark app to device..."
    # Add deployment commands here
    
elif [ "$PLATFORM" == "android" ]; then
    if ! command -v adb &> /dev/null; then
        echo "Error: adb not found. This script requires Android SDK."
        exit 1
    fi
    
    if [ -n "$DEVICE_TYPE" ]; then
        DEVICE_ID=$DEVICE_TYPE
    else
        DEVICE_ID=$(adb devices | grep -v "List" | grep "device" | head -1 | cut -f1)
    fi
    
    echo "Using Android device: $DEVICE_ID"
    
    # Deploy and run benchmark app
    echo "Deploying benchmark app to device..."
    adb -s $DEVICE_ID install -r zeo-benchmark.apk
    
    echo "Starting battery monitoring..."
    adb -s $DEVICE_ID shell dumpsys batterystats | grep "Battery: " > battery_before.txt
    
    echo "Running benchmarks..."
    adb -s $DEVICE_ID shell am start -n com.zeo.benchmark/.MainActivity
    
    echo "Waiting for benchmarks to complete..."
    sleep 60
    
    echo "Collecting results..."
    adb -s $DEVICE_ID pull /sdcard/zeo_benchmark_results.json ./reports/mobile_results.json
    
    echo "Collecting battery statistics..."
    adb -s $DEVICE_ID shell dumpsys batterystats | grep "Battery: " > battery_after.txt
    
    echo "Calculating battery consumption..."
    # Add battery calculation logic here
fi

echo "Mobile benchmarks completed!"
echo "Results saved to ./reports/mobile_results.json"