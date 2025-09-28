#!/bin/bash

# Test Supabase and Upstash connections

echo "Testing connections..."

# URL encode the password (! becomes %21)
ENCODED_PASSWORD="ypf6GHQ3mdp%21qhw0ywp"

# Test Supabase
echo "1. Testing Supabase connection..."
psql "postgresql://postgres:${ENCODED_PASSWORD}@db.zmzezcpmindmhpygtjle.supabase.co:5432/postgres?sslmode=require" -c "SELECT 'Supabase connected!' as status" 2>&1

# Test Upstash
echo -e "\n2. Testing Upstash connection..."
redis-cli -u "redis://default:AcE_AAIncDE5MGYxZWM5ZGNiYmY0OWU3OGQ2NjliODc5YjgyNDcxNHAxNDk0NzE@adjusted-goshawk-49471.upstash.io:6379" ping 2>&1

echo -e "\nConnection tests completed."