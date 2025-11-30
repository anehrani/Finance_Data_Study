#!/bin/bash

# Quick test script for data_streamer
# This will run for 30 seconds to verify WebSocket connections work

echo "Starting data_streamer test (will run for 30 seconds)..."
echo ""

# Run in background
cargo run &
PID=$!

# Wait 30 seconds
sleep 30

# Kill the process
echo ""
echo "Stopping data_streamer..."
kill $PID 2>/dev/null

# Show results
echo ""
echo "=== Test Results ==="
echo ""

if [ -d "tick_data/spot" ]; then
    echo "✓ Spot tick data directory created"
    SPOT_FILES=$(ls tick_data/spot/*.txt 2>/dev/null | wc -l)
    echo "  Files created: $SPOT_FILES"
    
    # Show sample data from first file
    FIRST_FILE=$(ls tick_data/spot/*.txt 2>/dev/null | head -1)
    if [ -f "$FIRST_FILE" ]; then
        LINES=$(wc -l < "$FIRST_FILE")
        echo "  Sample file: $(basename $FIRST_FILE) ($LINES ticks)"
        echo "  First 3 ticks:"
        head -3 "$FIRST_FILE" | sed 's/^/    /'
    fi
else
    echo "✗ No spot tick data created"
fi

echo ""

if [ -d "tick_data/linear" ]; then
    echo "✓ Linear tick data directory created"
    LINEAR_FILES=$(ls tick_data/linear/*.txt 2>/dev/null | wc -l)
    echo "  Files created: $LINEAR_FILES"
    
    # Show sample data from first file
    FIRST_FILE=$(ls tick_data/linear/*.txt 2>/dev/null | head -1)
    if [ -f "$FIRST_FILE" ]; then
        LINES=$(wc -l < "$FIRST_FILE")
        echo "  Sample file: $(basename $FIRST_FILE) ($LINES ticks)"
        echo "  First 3 ticks:"
        head -3 "$FIRST_FILE" | sed 's/^/    /'
    fi
else
    echo "✗ No linear tick data created"
fi

echo ""

if [ -d "bar_data/spot" ]; then
    echo "✓ Spot bar data directory created"
    BAR_FILES=$(ls bar_data/spot/*.txt 2>/dev/null | wc -l)
    echo "  Files created: $BAR_FILES"
else
    echo "✗ No spot bar data created"
fi

echo ""

if [ -d "historical_data/spot" ]; then
    echo "✓ Historical data downloaded"
    HIST_FILES=$(ls historical_data/spot/*.TXT 2>/dev/null | wc -l)
    echo "  Files created: $HIST_FILES"
else
    echo "✗ No historical data downloaded"
fi

echo ""
echo "Test complete!"
