#!/bin/bash
# Apache Bench Load Testing Script for Phase 9.4 Rate Limiting
# 
# Tests rate limiting enforcement using Apache Bench (ab)
#
# Prerequisites:
#   - Apache Bench installed: apt-get install apache2-utils
#   - API running on localhost:8000
#
# Usage:
#   bash ab_rate_limiting_test.sh
#   TEST_DURATION=300 bash ab_rate_limiting_test.sh  # 5 minute test

set -e

# Configuration
API_URL="${API_URL:-http://localhost:8000}"
TEST_DURATION="${TEST_DURATION:-60}"  # seconds
RESULTS_DIR="./load_test_results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Create results directory
mkdir -p "$RESULTS_DIR"

echo -e "${BLUE}=== Phase 9.4 Rate Limiting Load Test ===${NC}"
echo -e "${BLUE}Timestamp: $TIMESTAMP${NC}"
echo -e "${BLUE}API URL: $API_URL${NC}"
echo -e "${BLUE}Test Duration: ${TEST_DURATION}s${NC}"
echo ""

# Check if API is accessible
echo -n "Checking API health... "
if curl -sf "$API_URL/health" > /dev/null; then
    echo -e "${GREEN}✓ API is accessible${NC}"
else
    echo -e "${RED}✗ API is not accessible${NC}"
    exit 1
fi

# Test 1: Free Tier Rate Limiting (100 req/min)
echo -e "\n${YELLOW}Test 1: Free Tier - Single User (100 req/min limit)${NC}"
echo "Sending 150 requests in rapid succession..."

ab -n 150 -c 1 \
   -H "X-API-Key: test_user_free" \
   -H "X-Quota-Tier: free" \
   -H "Content-Type: application/json" \
   -p tests/load/payloads/identify.json \
   -g "$RESULTS_DIR/free_tier_${TIMESTAMP}.tsv" \
   "$API_URL/identify" \
   > "$RESULTS_DIR/free_tier_${TIMESTAMP}.txt" 2>&1

# Analyze results
SUCCESS_COUNT=$(grep -o "2xx responses:" "$RESULTS_DIR/free_tier_${TIMESTAMP}.txt" | head -1 | grep -o "[0-9]*" || echo "0")
RATE_LIMITED=$(grep -o "Non-2xx responses:" "$RESULTS_DIR/free_tier_${TIMESTAMP}.txt" | head -1 | grep -o "[0-9]*" || echo "0")

echo -e "Results:"
echo -e "  Successful (200): ${GREEN}$SUCCESS_COUNT${NC}"
echo -e "  Rate Limited (429): ${RED}$RATE_LIMITED${NC}"

if [ "$SUCCESS_COUNT" -ge 100 ] && [ "$SUCCESS_COUNT" -le 150 ]; then
    echo -e "  ${GREEN}✓ Free tier rate limiting working correctly${NC}"
else
    echo -e "  ${YELLOW}⚠ Expected ~100-150 successful, got $SUCCESS_COUNT${NC}"
fi

# Test 2: Pro Tier Rate Limiting (1000 req/min)
echo -e "\n${YELLOW}Test 2: Pro Tier - Single User (1000 req/min limit)${NC}"
echo "Sending 1200 requests..."

ab -n 1200 -c 10 \
   -H "X-API-Key: test_user_pro" \
   -H "X-Quota-Tier: pro" \
   -H "Content-Type: application/json" \
   -p tests/load/payloads/identify.json \
   -g "$RESULTS_DIR/pro_tier_${TIMESTAMP}.tsv" \
   "$API_URL/identify" \
   > "$RESULTS_DIR/pro_tier_${TIMESTAMP}.txt" 2>&1

SUCCESS_COUNT=$(grep -o "2xx responses:" "$RESULTS_DIR/pro_tier_${TIMESTAMP}.txt" | head -1 | grep -o "[0-9]*" || echo "0")
RATE_LIMITED=$(grep -o "Non-2xx responses:" "$RESULTS_DIR/pro_tier_${TIMESTAMP}.txt" | head -1 | grep -o "[0-9]*" || echo "0")

echo -e "Results:"
echo -e "  Successful (200): ${GREEN}$SUCCESS_COUNT${NC}"
echo -e "  Rate Limited (429): ${RED}$RATE_LIMITED${NC}"

if [ "$SUCCESS_COUNT" -ge 1000 ] && [ "$SUCCESS_COUNT" -le 1500 ]; then
    echo -e "  ${GREEN}✓ Pro tier rate limiting working correctly${NC}"
else
    echo -e "  ${YELLOW}⚠ Expected ~1000-1500 successful, got $SUCCESS_COUNT${NC}"
fi

# Test 3: Multiple Concurrent Users (Free Tier)
echo -e "\n${YELLOW}Test 3: Multiple Concurrent Users (10 users, Free tier)${NC}"
echo "Each user should independently get 100 req/min..."

for i in {1..10}; do
    ab -n 120 -c 1 \
       -H "X-API-Key: concurrent_user_$i" \
       -H "X-Quota-Tier: free" \
       -H "Content-Type: application/json" \
       -p tests/load/payloads/identify.json \
       "$API_URL/identify" \
       > "$RESULTS_DIR/concurrent_user${i}_${TIMESTAMP}.txt" 2>&1 &
done

wait

# Analyze concurrent user results
TOTAL_SUCCESS=0
TOTAL_RATE_LIMITED=0

for i in {1..10}; do
    SUCCESS=$(grep -o "2xx responses:" "$RESULTS_DIR/concurrent_user${i}_${TIMESTAMP}.txt" | head -1 | grep -o "[0-9]*" || echo "0")
    RATE_LIMITED=$(grep -o "Non-2xx responses:" "$RESULTS_DIR/concurrent_user${i}_${TIMESTAMP}.txt" | head -1 | grep -o "[0-9]*" || echo "0")
    TOTAL_SUCCESS=$((TOTAL_SUCCESS + SUCCESS))
    TOTAL_RATE_LIMITED=$((TOTAL_RATE_LIMITED + RATE_LIMITED))
done

echo -e "Results (10 users total):"
echo -e "  Total Successful: ${GREEN}$TOTAL_SUCCESS${NC}"
echo -e "  Total Rate Limited: ${RED}$TOTAL_RATE_LIMITED${NC}"
echo -e "  Average per user: $((TOTAL_SUCCESS / 10))"

if [ "$TOTAL_SUCCESS" -ge 1000 ]; then
    echo -e "  ${GREEN}✓ Concurrent users working independently${NC}"
else
    echo -e "  ${YELLOW}⚠ Expected ~1000+ successful total, got $TOTAL_SUCCESS${NC}"
fi

# Test 4: Compare Endpoint (2x cost)
echo -e "\n${YELLOW}Test 4: Compare Endpoint (2x cost)${NC}"
echo "Sending 120 requests (should hit limit at ~50-75 due to 2x cost)..."

ab -n 120 -c 1 \
   -H "X-API-Key: test_user_compare" \
   -H "X-Quota-Tier: free" \
   -H "Content-Type: application/json" \
   -p tests/load/payloads/compare.json \
   -g "$RESULTS_DIR/compare_${TIMESTAMP}.tsv" \
   "$API_URL/compare" \
   > "$RESULTS_DIR/compare_${TIMESTAMP}.txt" 2>&1

SUCCESS_COUNT=$(grep -o "2xx responses:" "$RESULTS_DIR/compare_${TIMESTAMP}.txt" | head -1 | grep -o "[0-9]*" || echo "0")
RATE_LIMITED=$(grep -o "Non-2xx responses:" "$RESULTS_DIR/compare_${TIMESTAMP}.txt" | head -1 | grep -o "[0-9]*" || echo "0")

echo -e "Results:"
echo -e "  Successful (200): ${GREEN}$SUCCESS_COUNT${NC}"
echo -e "  Rate Limited (429): ${RED}$RATE_LIMITED${NC}"

if [ "$SUCCESS_COUNT" -ge 50 ] && [ "$SUCCESS_COUNT" -le 75 ]; then
    echo -e "  ${GREEN}✓ Endpoint cost multiplier working correctly${NC}"
else
    echo -e "  ${YELLOW}⚠ Expected ~50-75 successful (2x cost), got $SUCCESS_COUNT${NC}"
fi

# Test 5: IP-Based Rate Limiting (unauthenticated)
echo -e "\n${YELLOW}Test 5: IP-Based Rate Limiting (30 req/min)${NC}"
echo "Sending 50 requests without authentication..."

ab -n 50 -c 1 \
   -H "Content-Type: application/json" \
   -p tests/load/payloads/identify.json \
   -g "$RESULTS_DIR/ip_based_${TIMESTAMP}.tsv" \
   "$API_URL/identify" \
   > "$RESULTS_DIR/ip_based_${TIMESTAMP}.txt" 2>&1

SUCCESS_COUNT=$(grep -o "2xx responses:" "$RESULTS_DIR/ip_based_${TIMESTAMP}.txt" | head -1 | grep -o "[0-9]*" || echo "0")
RATE_LIMITED=$(grep -o "Non-2xx responses:" "$RESULTS_DIR/ip_based_${TIMESTAMP}.txt" | head -1 | grep -o "[0-9]*" || echo "0")

echo -e "Results:"
echo -e "  Successful (200): ${GREEN}$SUCCESS_COUNT${NC}"
echo -e "  Rate Limited (429): ${RED}$RATE_LIMITED${NC}"

if [ "$SUCCESS_COUNT" -ge 30 ] && [ "$SUCCESS_COUNT" -le 45 ]; then
    echo -e "  ${GREEN}✓ IP-based rate limiting working correctly${NC}"
else
    echo -e "  ${YELLOW}⚠ Expected ~30-45 successful, got $SUCCESS_COUNT${NC}"
fi

# Test 6: Health Endpoint (should never be rate limited)
echo -e "\n${YELLOW}Test 6: Health Endpoint (exempt from rate limiting)${NC}"
echo "Sending 200 requests to health endpoint..."

ab -n 200 -c 10 \
   "$API_URL/health" \
   > "$RESULTS_DIR/health_${TIMESTAMP}.txt" 2>&1

SUCCESS_COUNT=$(grep -o "2xx responses:" "$RESULTS_DIR/health_${TIMESTAMP}.txt" | head -1 | grep -o "[0-9]*" || echo "0")
RATE_LIMITED=$(grep -o "Non-2xx responses:" "$RESULTS_DIR/health_${TIMESTAMP}.txt" | head -1 | grep -o "[0-9]*" || echo "0")

echo -e "Results:"
echo -e "  Successful (200): ${GREEN}$SUCCESS_COUNT${NC}"
echo -e "  Rate Limited (429): ${RED}$RATE_LIMITED${NC}"

if [ "$SUCCESS_COUNT" == "200" ] && [ "$RATE_LIMITED" == "0" ]; then
    echo -e "  ${GREEN}✓ Health endpoint correctly exempt from rate limiting${NC}"
else
    echo -e "  ${RED}✗ Health endpoint should never be rate limited${NC}"
fi

# Test 7: Sustained Load (Free Tier)
echo -e "\n${YELLOW}Test 7: Sustained Load (60 seconds at 120 req/min)${NC}"
echo "Testing token bucket refill over time..."

ab -t "$TEST_DURATION" -c 2 -n 999999 \
   -H "X-API-Key: test_user_sustained" \
   -H "X-Quota-Tier: free" \
   -H "Content-Type: application/json" \
   -p tests/load/payloads/identify.json \
   -g "$RESULTS_DIR/sustained_${TIMESTAMP}.tsv" \
   "$API_URL/identify" \
   > "$RESULTS_DIR/sustained_${TIMESTAMP}.txt" 2>&1

SUCCESS_COUNT=$(grep -o "2xx responses:" "$RESULTS_DIR/sustained_${TIMESTAMP}.txt" | head -1 | grep -o "[0-9]*" || echo "0")
RATE_LIMITED=$(grep -o "Non-2xx responses:" "$RESULTS_DIR/sustained_${TIMESTAMP}.txt" | head -1 | grep -o "[0-9]*" || echo "0")
REQUESTS_PER_SEC=$(grep "Requests per second:" "$RESULTS_DIR/sustained_${TIMESTAMP}.txt" | grep -o "[0-9.]*" | head -1)

echo -e "Results:"
echo -e "  Successful (200): ${GREEN}$SUCCESS_COUNT${NC}"
echo -e "  Rate Limited (429): ${RED}$RATE_LIMITED${NC}"
echo -e "  Requests per second: ${BLUE}$REQUESTS_PER_SEC${NC}"

# For 60s test at 100 req/min, expect ~100 successful
EXPECTED_SUCCESS=$((100 * TEST_DURATION / 60))
if [ "$SUCCESS_COUNT" -ge $((EXPECTED_SUCCESS - 20)) ] && [ "$SUCCESS_COUNT" -le $((EXPECTED_SUCCESS + 20)) ]; then
    echo -e "  ${GREEN}✓ Token bucket refill working correctly${NC}"
else
    echo -e "  ${YELLOW}⚠ Expected ~$EXPECTED_SUCCESS successful, got $SUCCESS_COUNT${NC}"
fi

# Fetch metrics from API
echo -e "\n${YELLOW}Fetching Rate Limit Metrics from API...${NC}"
curl -s "$API_URL/api/v1/rate-limit/metrics/json" | jq '.' > "$RESULTS_DIR/metrics_${TIMESTAMP}.json" 2>/dev/null || echo "Could not fetch metrics"

# Summary
echo -e "\n${BLUE}=== Test Summary ===${NC}"
echo -e "All test results saved to: ${GREEN}$RESULTS_DIR${NC}"
echo ""
echo "Result files:"
ls -lh "$RESULTS_DIR"/*${TIMESTAMP}* 2>/dev/null | awk '{print "  " $9 " (" $5 ")"}'

echo -e "\n${BLUE}=== Next Steps ===${NC}"
echo "1. Review detailed results in $RESULTS_DIR"
echo "2. Check Grafana dashboard for visualization"
echo "3. View Prometheus metrics: curl $API_URL/api/v1/rate-limit/metrics"
echo "4. Analyze response time distribution in .tsv files"
echo ""
echo -e "${GREEN}Load test complete!${NC}"
