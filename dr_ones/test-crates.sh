#!/bin/bash

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored status messages
print_status() {
    local color=$1
    local message=$2
    echo -e "${color}${message}${NC}"
}

# Function to handle errors
handle_error() {
    print_status "${RED}" "Error: $1"
    exit 1
}

# Check if we're in a git repository
if ! git rev-parse --is-inside-work-tree > /dev/null 2>&1; then
    handle_error "Not in a git repository"
fi

# Array of crate directories in dependency order
CRATES=(
    "common"
    "drone"
    "client"
    "server"
    "simulation_controller"
    "network_initializer"
)

# Variables for test summary
TOTAL_TESTS=0
FAILED_TESTS=0
FAILED_CRATES=()

# Function to process each crate
test_crate() {
    local crate=$1
    print_status "${YELLOW}" "\nTesting crate: ${crate}"
    
    # Check if directory exists
    if [ ! -d "$crate" ]; then
        print_status "${RED}" "Directory $crate not found, skipping..."
        return
    fi
    
    # Change to crate directory
    cd "$crate" || handle_error "Failed to enter $crate directory"
    
    # Run cargo test and capture output
    OUTPUT=$(cargo test 2>&1)
    EXIT_CODE=$?
    
    # Extract test summary numbers using a more robust method
    if echo "$OUTPUT" | grep -q "running.*tests"; then
        # Count total tests from the "running X tests" line
        local test_count=$(echo "$OUTPUT" | grep "running.*test" | awk '{sum += $2} END {print sum}')
        TOTAL_TESTS=$((TOTAL_TESTS + test_count))
        
        # Check if any tests failed
        if [ $EXIT_CODE -ne 0 ]; then
            # Parse failed tests count from "test result: FAILED. X passed; Y failed" line
            local failed_count=$(echo "$OUTPUT" | grep "test result: FAILED" | grep -o '[0-9]* failed' | awk '{sum += $1} END {print sum}')
            FAILED_TESTS=$((FAILED_TESTS + failed_count))
            FAILED_CRATES+=("$crate")
            print_status "${RED}" "Tests failed in $crate"
            echo "$OUTPUT"
        else
            print_status "${GREEN}" "All tests passed in $crate ($test_count tests)"
        fi
    else
        print_status "${YELLOW}" "No tests found in $crate"
    fi
    
    # Return to root directory
    cd - > /dev/null || handle_error "Failed to return to root directory"
}

# Print header
print_status "${GREEN}" "Starting test suite execution..."

# Process all crates
for crate in "${CRATES[@]}"; do
    test_crate "$crate"
done

# Print summary
echo -e "\n----------------------------------------"
print_status "${GREEN}" "Test Suite Summary:"
echo "Total tests executed: $TOTAL_TESTS"
if [ ${#FAILED_CRATES[@]} -eq 0 ]; then
    print_status "${GREEN}" "All tests passed successfully! 🎉"
else
    print_status "${RED}" "Failed tests: $FAILED_TESTS"
    print_status "${RED}" "Crates with failures:"
    for crate in "${FAILED_CRATES[@]}"; do
        echo "  - $crate"
    done
    exit 1
fi