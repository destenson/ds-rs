#!/bin/bash

# Test Orchestrator for ds-rs project (Shell version)
# Manages test scenarios, RTSP servers, and test environments

set -e  # Exit on error

# Script configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
CONFIG_FILE="${CONFIG_FILE:-$SCRIPT_DIR/config/test-scenarios.toml}"
SCENARIO="${1:-all}"
VERBOSE=${VERBOSE:-0}
DRY_RUN=${DRY_RUN:-0}

# Process tracking
declare -A PROCESSES
declare -a TEST_RESULTS

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Cleanup function
cleanup() {
    echo -e "${YELLOW}Cleaning up processes...${NC}"
    for pid in "${!PROCESSES[@]}"; do
        stop_process "$pid"
    done
}

# Register cleanup on exit
trap cleanup EXIT INT TERM

# Logging functions
log_info() {
    echo -e "${CYAN}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[✓]${NC} $1"
}

log_error() {
    echo -e "${RED}[✗]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[⚠]${NC} $1"
}

log_verbose() {
    if [ "$VERBOSE" -eq 1 ]; then
        echo -e "${CYAN}[DEBUG]${NC} $1"
    fi
}

# Parse TOML configuration (basic parser using Python)
parse_toml() {
    local config_file="$1"
    
    if command -v python3 &> /dev/null; then
        python3 -c "
import tomli
import json
with open('$config_file', 'rb') as f:
    config = tomli.load(f)
print(json.dumps(config))
" 2>/dev/null
    else
        log_error "Python3 with tomli required for configuration parsing"
        log_info "Install with: pip3 install tomli"
        exit 1
    fi
}

# Check if TCP port is available
check_port() {
    local host="${1:-127.0.0.1}"
    local port="$2"
    local timeout="${3:-5}"
    
    timeout "$timeout" bash -c "cat < /dev/null > /dev/tcp/$host/$port" 2>/dev/null
    return $?
}

# Start a background process
start_process() {
    local name="$1"
    local command="$2"
    local cwd="${3:-$PROJECT_ROOT}"
    
    log_info "Starting process: $name"
    log_verbose "Command: $command"
    log_verbose "CWD: $cwd"
    
    # Start process in background
    (cd "$cwd" && eval "$command" > "/tmp/${name}.out" 2> "/tmp/${name}.err") &
    local pid=$!
    
    PROCESSES["$name"]=$pid
    log_verbose "Started process $name with PID $pid"
    
    return 0
}

# Stop a background process
stop_process() {
    local name="$1"
    
    if [ -n "${PROCESSES[$name]}" ]; then
        local pid="${PROCESSES[$name]}"
        log_info "Stopping process: $name (PID: $pid)"
        
        # Try graceful shutdown first
        if kill -0 "$pid" 2>/dev/null; then
            kill -TERM "$pid" 2>/dev/null || true
            
            # Wait for process to terminate
            local count=0
            while kill -0 "$pid" 2>/dev/null && [ $count -lt 5 ]; do
                sleep 1
                count=$((count + 1))
            done
            
            # Force kill if still running
            if kill -0 "$pid" 2>/dev/null; then
                log_warning "Force killing process $name"
                kill -9 "$pid" 2>/dev/null || true
            fi
        fi
        
        unset PROCESSES["$name"]
    fi
}

# Wait for health check
wait_for_health_check() {
    local host="${1:-127.0.0.1}"
    local port="$2"
    local timeout="${3:-10}"
    
    log_info "Waiting for $host:$port to be available..."
    
    local elapsed=0
    while [ $elapsed -lt "$timeout" ]; do
        if check_port "$host" "$port" 1; then
            log_success "Port $host:$port is available"
            return 0
        fi
        sleep 1
        elapsed=$((elapsed + 1))
    done
    
    log_error "Timeout waiting for $host:$port"
    return 1
}

# Run a test command
run_command() {
    local name="$1"
    local command="$2"
    local cwd="${3:-$PROJECT_ROOT}"
    local expected_exit="${4:-0}"
    local retry_count="${5:-0}"
    local allow_failure="${6:-false}"
    local timeout="${7:-300}"
    
    log_info "Running: $name"
    log_verbose "Command: $command"
    log_verbose "CWD: $cwd"
    
    local attempt=0
    while [ $attempt -le "$retry_count" ]; do
        if [ $attempt -gt 0 ]; then
            log_warning "Retry attempt $attempt/$retry_count"
        fi
        
        # Run command with timeout
        local output_file="/tmp/test-output-$$.txt"
        local error_file="/tmp/test-error-$$.txt"
        
        if timeout "$timeout" bash -c "cd '$cwd' && $command" > "$output_file" 2> "$error_file"; then
            local exit_code=$?
            
            if [ $exit_code -eq "$expected_exit" ]; then
                log_success "$name completed successfully"
                rm -f "$output_file" "$error_file"
                return 0
            elif [ "$allow_failure" = "true" ]; then
                log_warning "$name failed but marked as allow_failure"
                rm -f "$output_file" "$error_file"
                return 0
            else
                log_error "$name failed with exit code $exit_code"
                if [ "$VERBOSE" -eq 1 ]; then
                    echo "stdout:" && cat "$output_file"
                    echo "stderr:" && cat "$error_file"
                fi
            fi
        else
            log_error "$name failed or timed out"
        fi
        
        rm -f "$output_file" "$error_file"
        attempt=$((attempt + 1))
    done
    
    return 1
}

# Setup test scenario
setup_scenario() {
    local scenario_json="$1"
    
    # Parse setup configuration
    local rtsp_enabled=$(echo "$scenario_json" | jq -r '.setup.rtsp_server.enabled // false')
    
    # Start RTSP server if needed
    if [ "$rtsp_enabled" = "true" ]; then
        local rtsp_command=$(echo "$scenario_json" | jq -r '.setup.rtsp_server.command')
        local rtsp_cwd=$(echo "$scenario_json" | jq -r '.setup.rtsp_server.cwd')
        local startup_delay=$(echo "$scenario_json" | jq -r '.setup.rtsp_server.startup_delay // 5')
        
        start_process "rtsp_server" "$rtsp_command" "$PROJECT_ROOT/$rtsp_cwd"
        
        log_info "Waiting $startup_delay seconds for RTSP server to start..."
        sleep "$startup_delay"
        
        # Health check
        local health_port=$(echo "$scenario_json" | jq -r '.setup.rtsp_server.health_check.port // 8554')
        local health_timeout=$(echo "$scenario_json" | jq -r '.setup.rtsp_server.health_check.timeout // 10')
        
        if ! wait_for_health_check "127.0.0.1" "$health_port" "$health_timeout"; then
            log_error "RTSP server health check failed"
            return 1
        fi
    fi
    
    # Create test files if needed
    local test_files_enabled=$(echo "$scenario_json" | jq -r '.setup.test_files.enabled // false')
    if [ "$test_files_enabled" = "true" ]; then
        local files=$(echo "$scenario_json" | jq -c '.setup.test_files.files[]')
        
        while IFS= read -r file_config; do
            local pattern=$(echo "$file_config" | jq -r '.pattern')
            local duration=$(echo "$file_config" | jq -r '.duration')
            local output=$(echo "$file_config" | jq -r '.output')
            
            log_info "Generating test file: $output"
            
            local generate_cmd="cargo run -- generate --pattern $pattern --duration $duration --output $output"
            if ! run_command "Generate $output" "$generate_cmd" "$PROJECT_ROOT/crates/source-videos" 0 0 false 60; then
                log_error "Failed to generate test file: $output"
                return 1
            fi
        done <<< "$files"
    fi
    
    return 0
}

# Cleanup test scenario
cleanup_scenario() {
    local scenario_json="$1"
    
    # Stop RTSP server
    local stop_rtsp=$(echo "$scenario_json" | jq -r '.cleanup.stop_rtsp_server // false')
    if [ "$stop_rtsp" = "true" ]; then
        stop_process "rtsp_server"
    fi
    
    # Remove test files
    local remove_files=$(echo "$scenario_json" | jq -r '.cleanup.remove_test_files // false')
    if [ "$remove_files" = "true" ]; then
        local test_files=("test_smpte.mp4" "test_ball.mp4")
        for file in "${test_files[@]}"; do
            local file_path="$PROJECT_ROOT/crates/ds-rs/$file"
            if [ -f "$file_path" ]; then
                log_info "Removing test file: $file"
                rm -f "$file_path"
            fi
        done
    fi
}

# Run test scenario
run_scenario() {
    local scenario_name="$1"
    local config_json="$2"
    
    # Get scenario configuration
    local scenario_json=$(echo "$config_json" | jq -c ".scenarios.\"$scenario_name\"")
    
    if [ "$scenario_json" = "null" ]; then
        log_error "Scenario '$scenario_name' not found"
        return 1
    fi
    
    local description=$(echo "$scenario_json" | jq -r '.description // "No description"')
    
    echo
    echo "============================================================"
    log_info "Running scenario: $scenario_name"
    log_info "Description: $description"
    echo "============================================================"
    echo
    
    # Check if this scenario includes other scenarios
    local include_scenarios=$(echo "$scenario_json" | jq -r '.include_scenarios[]?' 2>/dev/null)
    if [ -n "$include_scenarios" ]; then
        local success=true
        while IFS= read -r included; do
            if ! run_scenario "$included" "$config_json"; then
                success=false
                local continue_on_failure=$(echo "$scenario_json" | jq -r '.continue_on_failure // false')
                if [ "$continue_on_failure" != "true" ]; then
                    return 1
                fi
            fi
        done <<< "$include_scenarios"
        
        if [ "$success" = "true" ]; then
            return 0
        else
            return 1
        fi
    fi
    
    # Setup
    if ! setup_scenario "$scenario_json"; then
        log_error "Scenario setup failed"
        cleanup_scenario "$scenario_json"
        return 1
    fi
    
    # Get environment variables
    local env_vars=$(echo "$scenario_json" | jq -r '.env // {} | to_entries | .[] | "\(.key)=\(.value)"')
    
    # Export environment variables
    while IFS= read -r env_var; do
        if [ -n "$env_var" ]; then
            export "$env_var"
            log_verbose "Set environment: $env_var"
        fi
    done <<< "$env_vars"
    
    # Run steps
    local success=true
    local steps=$(echo "$scenario_json" | jq -c '.steps[]?' 2>/dev/null)
    
    if [ -n "$steps" ]; then
        while IFS= read -r step; do
            local step_name=$(echo "$step" | jq -r '.name // "unnamed"')
            local step_command=$(echo "$step" | jq -r '.command')
            local step_cwd=$(echo "$step" | jq -r '.cwd // "."')
            local expected_exit=$(echo "$step" | jq -r '.expected_exit_code // 0')
            local retry_count=$(echo "$step" | jq -r '.retry_count // 0')
            local allow_failure=$(echo "$step" | jq -r '.allow_failure // false')
            local step_timeout=$(echo "$step" | jq -r '.timeout // 300')
            
            # Apply step-specific environment
            local step_env=$(echo "$step" | jq -r '.env // {} | to_entries | .[] | "\(.key)=\(.value)"')
            while IFS= read -r env_var; do
                if [ -n "$env_var" ]; then
                    export "$env_var"
                fi
            done <<< "$step_env"
            
            if ! run_command "$step_name" "$step_command" "$PROJECT_ROOT/$step_cwd" \
                "$expected_exit" "$retry_count" "$allow_failure" "$step_timeout"; then
                success=false
                local continue_on_failure=$(echo "$scenario_json" | jq -r '.continue_on_failure // false')
                if [ "$continue_on_failure" != "true" ]; then
                    break
                fi
            fi
        done <<< "$steps"
    fi
    
    # Cleanup
    cleanup_scenario "$scenario_json"
    
    # Report results
    if [ "$success" = "true" ]; then
        log_success "Scenario '$scenario_name' completed successfully"
        TEST_RESULTS+=("$scenario_name:PASSED")
    else
        log_error "Scenario '$scenario_name' failed"
        TEST_RESULTS+=("$scenario_name:FAILED")
    fi
    
    return $([ "$success" = "true" ] && echo 0 || echo 1)
}

# List available scenarios
list_scenarios() {
    local config_json="$1"
    
    echo
    log_info "Available test scenarios:"
    echo "------------------------------------------------------------"
    
    echo "$config_json" | jq -r '.scenarios | to_entries | .[] | "  \(.key | .[0:20] | . + (" " * (20 - length))) - \(.value.description // "No description")"'
    
    echo
}

# Print test summary
print_summary() {
    echo
    echo "============================================================"
    log_info "TEST EXECUTION SUMMARY"
    echo "============================================================"
    
    local total=${#TEST_RESULTS[@]}
    local passed=0
    local failed=0
    
    for result in "${TEST_RESULTS[@]}"; do
        local scenario="${result%%:*}"
        local status="${result##*:}"
        
        if [ "$status" = "PASSED" ]; then
            echo -e "  ${scenario}$(printf '%*s' $((20 - ${#scenario})) '') - ${GREEN}✓ PASSED${NC}"
            passed=$((passed + 1))
        else
            echo -e "  ${scenario}$(printf '%*s' $((20 - ${#scenario})) '') - ${RED}✗ FAILED${NC}"
            failed=$((failed + 1))
        fi
    done
    
    echo "------------------------------------------------------------"
    echo "Total: $total, Passed: $passed, Failed: $failed"
    
    if [ $failed -eq 0 ]; then
        log_success "All tests passed!"
        return 0
    else
        log_error "$failed test(s) failed"
        return 1
    fi
}

# Show usage
usage() {
    cat << EOF
Usage: $0 [OPTIONS] [SCENARIO]

Test Orchestrator for ds-rs project

Arguments:
  SCENARIO          Test scenario to run (default: all)

Options:
  -c, --config FILE Configuration file (default: scripts/config/test-scenarios.toml)
  -l, --list        List available test scenarios
  -v, --verbose     Enable verbose output
  -d, --dry-run     Show what would be executed without running tests
  -h, --help        Show this help message

Examples:
  $0 unit           Run unit tests
  $0 integration    Run integration tests
  $0 -l             List all scenarios
  $0 -v e2e         Run end-to-end tests with verbose output

Environment Variables:
  VERBOSE=1         Enable verbose output
  DRY_RUN=1         Dry run mode
  CONFIG_FILE       Path to configuration file
EOF
}

# Main execution
main() {
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -c|--config)
                CONFIG_FILE="$2"
                shift 2
                ;;
            -l|--list)
                LIST_ONLY=1
                shift
                ;;
            -v|--verbose)
                VERBOSE=1
                shift
                ;;
            -d|--dry-run)
                DRY_RUN=1
                shift
                ;;
            -h|--help)
                usage
                exit 0
                ;;
            -*)
                log_error "Unknown option: $1"
                usage
                exit 1
                ;;
            *)
                SCENARIO="$1"
                shift
                ;;
        esac
    done
    
    # Check for required tools
    if ! command -v jq &> /dev/null; then
        log_error "jq is required but not installed"
        log_info "Install with: apt-get install jq (Debian/Ubuntu) or brew install jq (macOS)"
        exit 1
    fi
    
    if ! command -v python3 &> /dev/null; then
        log_error "Python3 is required but not installed"
        exit 1
    fi
    
    # Check if tomli is installed
    if ! python3 -c "import tomli" 2>/dev/null; then
        log_error "Python tomli package is required"
        log_info "Install with: pip3 install tomli"
        exit 1
    fi
    
    # Check configuration file
    if [ ! -f "$CONFIG_FILE" ]; then
        log_error "Configuration file not found: $CONFIG_FILE"
        exit 1
    fi
    
    # Parse configuration
    log_verbose "Loading configuration from: $CONFIG_FILE"
    CONFIG_JSON=$(parse_toml "$CONFIG_FILE")
    
    if [ -z "$CONFIG_JSON" ]; then
        log_error "Failed to parse configuration file"
        exit 1
    fi
    
    # List scenarios if requested
    if [ "${LIST_ONLY:-0}" -eq 1 ]; then
        list_scenarios "$CONFIG_JSON"
        exit 0
    fi
    
    # Dry run mode
    if [ "$DRY_RUN" -eq 1 ]; then
        log_warning "DRY RUN MODE - Commands will not be executed"
    fi
    
    # Run scenario
    if run_scenario "$SCENARIO" "$CONFIG_JSON"; then
        print_summary
        exit 0
    else
        print_summary
        exit 1
    fi
}

# Run main function
main "$@"