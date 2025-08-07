#!/bin/sh
# Cron job helper script with retry logic and error handling

# Configuration
MAX_RETRIES=3
RETRY_DELAY=5
BACKEND_URL="http://back:8000"

# Function to execute HTTP request with retries
execute_with_retry() {
    local method=$1
    local endpoint=$2
    local job_name=$3
    local retry_count=0
    
    while [ $retry_count -lt $MAX_RETRIES ]; do
        echo "[$(date +'%Y-%m-%d %H:%M:%S')] $job_name: Attempt $((retry_count + 1))/$MAX_RETRIES"
        
        # Execute curl command
        response=$(curl -s -w "\n%{http_code}" -X "$method" "$BACKEND_URL$endpoint" 2>&1)
        http_code=$(echo "$response" | tail -n1)
        body=$(echo "$response" | sed '$d')
        
        # Check if request was successful (2xx status code)
        if [ "$http_code" -ge 200 ] && [ "$http_code" -lt 300 ]; then
            echo "[$(date +'%Y-%m-%d %H:%M:%S')] $job_name: Success (HTTP $http_code)"
            if [ -n "$body" ]; then
                echo "[$(date +'%Y-%m-%d %H:%M:%S')] $job_name: Response: $body"
            fi
            return 0
        fi
        
        # Request failed, log error and retry
        echo "[$(date +'%Y-%m-%d %H:%M:%S')] $job_name: Failed (HTTP $http_code)" >&2
        if [ -n "$body" ]; then
            echo "[$(date +'%Y-%m-%d %H:%M:%S')] $job_name: Error: $body" >&2
        fi
        
        retry_count=$((retry_count + 1))
        
        if [ $retry_count -lt $MAX_RETRIES ]; then
            echo "[$(date +'%Y-%m-%d %H:%M:%S')] $job_name: Retrying in $RETRY_DELAY seconds..."
            sleep $RETRY_DELAY
        fi
    done
    
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] $job_name: Failed after $MAX_RETRIES attempts" >&2
    return 1
}

# Function for session sync
sync_sessions() {
    execute_with_retry "PUT" "/session" "SessionSync"
}

# Function for status history update
update_status_history() {
    execute_with_retry "PUT" "/status_history" "StatusUpdate"
}

# Function for cleanup expired sessions
cleanup_expired_sessions() {
    execute_with_retry "DELETE" "/session/expired" "SessionCleanup"
}

# Health check function
health_check() {
    local response=$(curl -s -w "\n%{http_code}" "$BACKEND_URL/health" 2>&1)
    local http_code=$(echo "$response" | tail -n1)
    
    if [ "$http_code" -eq 200 ]; then
        echo "[$(date +'%Y-%m-%d %H:%M:%S')] HealthCheck: Backend is healthy"
        return 0
    else
        echo "[$(date +'%Y-%m-%d %H:%M:%S')] HealthCheck: Backend is unhealthy (HTTP $http_code)" >&2
        return 1
    fi
}

# Main execution based on command argument
case "$1" in
    sync)
        sync_sessions
        ;;
    status)
        update_status_history
        ;;
    cleanup)
        cleanup_expired_sessions
        ;;
    health)
        health_check
        ;;
    *)
        echo "Usage: $0 {sync|status|cleanup|health}"
        exit 1
        ;;
esac