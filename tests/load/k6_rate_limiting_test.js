/**
 * k6 Load Testing Script for Phase 9.4 Rate Limiting
 * 
 * Tests rate limiting enforcement across different user tiers and endpoints.
 * 
 * Usage:
 *   k6 run k6_rate_limiting_test.js
 *   k6 run --vus 50 --duration 3m k6_rate_limiting_test.js
 *   k6 run --vus 100 --duration 5m --out json=results.json k6_rate_limiting_test.js
 */

import http from 'k6/http';
import { check, group, sleep } from 'k6';
import { Rate, Counter, Trend } from 'k6/metrics';

// Custom metrics
const rateLimitErrors = new Counter('rate_limit_errors');
const successfulRequests = new Counter('successful_requests');
const responseTimeP95 = new Trend('response_time_p95');
const rateLimitHeaderPresent = new Rate('rate_limit_header_present');

// Configuration
const BASE_URL = __ENV.API_URL || 'http://localhost:8000';
const TEST_DURATION = __ENV.TEST_DURATION || '3m';
const TARGET_VUS = __ENV.TARGET_VUS || 50;

// Load test stages
export const options = {
  stages: [
    { duration: '30s', target: 10 },      // Warm-up: ramp to 10 VUs
    { duration: '1m', target: TARGET_VUS }, // Ramp up to target
    { duration: TEST_DURATION, target: TARGET_VUS }, // Sustained load
    { duration: '30s', target: 0 },       // Ramp down
  ],
  thresholds: {
    http_req_duration: ['p(95)<500'],     // 95% of requests should be < 500ms
    http_req_failed: ['rate<0.5'],        // Less than 50% failure rate (accounting for rate limits)
    rate_limit_errors: ['count>0'],       // Expect some rate limit errors (normal behavior)
    successful_requests: ['count>100'],   // At least 100 successful requests
  },
};

// User tiers and their characteristics
const USER_TIERS = {
  free: {
    tier: 'free',
    minuteLimit: 100,
    monthlyQuota: 50000,
  },
  pro: {
    tier: 'pro',
    minuteLimit: 1000,
    monthlyQuota: 1000000,
  },
  enterprise: {
    tier: 'enterprise',
    minuteLimit: null, // unlimited
    monthlyQuota: null, // unlimited
  },
};

// Endpoints and their costs
const ENDPOINTS = {
  identify: { path: '/identify', cost: 1.0, method: 'POST' },
  compare: { path: '/compare', cost: 2.0, method: 'POST' },
  batch: { path: '/batch', cost: 1.0, method: 'POST' },
  health: { path: '/health', cost: 0.0, method: 'GET' },
};

/**
 * Generate unique user ID for this VU
 */
function getUserId() {
  return `user_vu${__VU}_iter${__ITER}`;
}

/**
 * Select user tier based on VU distribution
 */
function getUserTier() {
  const vu = __VU;
  
  // 70% free tier, 25% pro tier, 5% enterprise
  if (vu % 20 === 0) {
    return USER_TIERS.enterprise;
  } else if (vu % 4 === 0) {
    return USER_TIERS.pro;
  } else {
    return USER_TIERS.free;
  }
}

/**
 * Make request to API with rate limiting headers
 */
function makeRequest(endpoint, tier, userId) {
  const url = `${BASE_URL}${endpoint.path}`;
  const params = {
    headers: {
      'Content-Type': 'application/json',
      'X-API-Key': userId,
      'X-Quota-Tier': tier.tier,
    },
    tags: {
      endpoint: endpoint.path,
      tier: tier.tier,
    },
  };

  let response;
  
  if (endpoint.method === 'GET') {
    response = http.get(url, params);
  } else {
    const body = JSON.stringify({
      fingerprint: {
        ja4: 't13d1517h2_8daaf6152771_e5627efa2ab1',
        tls_version: 'TLS 1.3',
        cipher_suites: 17,
      },
    });
    response = http.post(url, body, params);
  }

  return response;
}

/**
 * Validate response headers and rate limit information
 */
function validateResponse(response, tier) {
  const checksPass = check(response, {
    'status is 200 or 429': (r) => r.status === 200 || r.status === 429,
    'has rate limit remaining header': (r) => 
      r.headers['X-Ratelimit-Remaining'] !== undefined ||
      r.headers['x-ratelimit-remaining'] !== undefined,
    'has rate limit reset header': (r) => 
      r.headers['X-Ratelimit-Reset'] !== undefined ||
      r.headers['x-ratelimit-reset'] !== undefined,
  });

  // Track custom metrics
  if (response.status === 200) {
    successfulRequests.add(1);
  } else if (response.status === 429) {
    rateLimitErrors.add(1);
  }

  // Track response time
  responseTimeP95.add(response.timings.duration);

  // Check for rate limit headers
  const hasRateLimitHeaders = 
    response.headers['X-Ratelimit-Remaining'] !== undefined ||
    response.headers['x-ratelimit-remaining'] !== undefined;
  rateLimitHeaderPresent.add(hasRateLimitHeaders ? 1 : 0);

  return checksPass;
}

/**
 * Main test scenario
 */
export default function () {
  const userId = getUserId();
  const tier = getUserTier();

  group('Rate Limiting Tests', () => {
    // Test 1: Identify endpoint (1.0x cost)
    group('Identify Endpoint', () => {
      const response = makeRequest(ENDPOINTS.identify, tier, userId);
      validateResponse(response, tier);
    });

    sleep(1); // Small delay between requests

    // Test 2: Compare endpoint (2.0x cost)
    group('Compare Endpoint', () => {
      const response = makeRequest(ENDPOINTS.compare, tier, userId);
      validateResponse(response, tier);
    });

    sleep(1);

    // Test 3: Health endpoint (0.0x cost, should never be rate limited)
    group('Health Endpoint', () => {
      const response = makeRequest(ENDPOINTS.health, tier, userId);
      check(response, {
        'health check always succeeds': (r) => r.status === 200,
      });
    });
  });

  // Random sleep to simulate realistic traffic patterns
  sleep(Math.random() * 3 + 1); // 1-4 seconds
}

/**
 * Burst test scenario - test burst capacity (1.5x)
 */
export function burstTest() {
  const userId = `burst_user_${__VU}`;
  const tier = USER_TIERS.free; // Test with free tier (100/min)

  console.log(`[VU ${__VU}] Starting burst test for user ${userId}`);

  // Send rapid requests to test burst capacity
  // Free tier: 100/min with 1.5x burst = 150 capacity
  const burstSize = 120; // Slightly above limit but within burst
  let successCount = 0;
  let rateLimitCount = 0;

  for (let i = 0; i < burstSize; i++) {
    const response = makeRequest(ENDPOINTS.identify, tier, userId);
    
    if (response.status === 200) {
      successCount++;
    } else if (response.status === 429) {
      rateLimitCount++;
    }

    // No sleep - burst as fast as possible
  }

  console.log(`[VU ${__VU}] Burst test results: ${successCount} successful, ${rateLimitCount} rate limited`);
  
  check(null, {
    'burst allows more than base limit': () => successCount > tier.minuteLimit,
    'burst capacity respected': () => successCount <= tier.minuteLimit * 1.5,
  });
}

/**
 * Tier comparison scenario
 */
export function tierComparisonTest() {
  const baseUserId = `tier_test_${__VU}`;

  Object.entries(USER_TIERS).forEach(([tierName, tier]) => {
    group(`${tierName} tier test`, () => {
      const userId = `${baseUserId}_${tierName}`;
      
      // Make requests up to the limit
      const requestCount = tier.minuteLimit || 50; // Use 50 for unlimited tiers
      let successCount = 0;
      let rateLimitCount = 0;

      for (let i = 0; i < requestCount; i++) {
        const response = makeRequest(ENDPOINTS.identify, tier, userId);
        
        if (response.status === 200) {
          successCount++;
        } else if (response.status === 429) {
          rateLimitCount++;
          break; // Stop on first rate limit
        }

        sleep(0.1); // Small delay
      }

      console.log(`[${tierName}] ${successCount} successful before rate limit`);

      // Unlimited tiers should never hit rate limit
      if (tier.minuteLimit === null) {
        check(null, {
          'unlimited tier never rate limited': () => rateLimitCount === 0,
        });
      }
    });
  });
}

/**
 * Setup function - runs once per VU
 */
export function setup() {
  // Verify API is accessible
  const healthCheck = http.get(`${BASE_URL}/health`);
  
  if (healthCheck.status !== 200) {
    throw new Error(`API health check failed: ${healthCheck.status}`);
  }

  console.log('✅ API health check passed');
  console.log(`📊 Starting load test with ${TARGET_VUS} VUs for ${TEST_DURATION}`);
  
  return {
    startTime: new Date().toISOString(),
  };
}

/**
 * Teardown function - runs once at the end
 */
export function teardown(data) {
  console.log('');
  console.log('=== Load Test Summary ===');
  console.log(`Started: ${data.startTime}`);
  console.log(`Ended: ${new Date().toISOString()}`);
  console.log('');
  console.log('Check metrics in Grafana dashboard or run:');
  console.log(`  curl ${BASE_URL}/api/v1/rate-limit/metrics`);
}

/**
 * Handle summary - custom summary output
 */
export function handleSummary(data) {
  const successful = data.metrics.successful_requests.values.count || 0;
  const rateLimited = data.metrics.rate_limit_errors.values.count || 0;
  const total = successful + rateLimited;
  const successRate = total > 0 ? (successful / total * 100).toFixed(2) : 0;

  console.log('');
  console.log('=== Rate Limiting Statistics ===');
  console.log(`Total Requests: ${total}`);
  console.log(`Successful (200): ${successful} (${successRate}%)`);
  console.log(`Rate Limited (429): ${rateLimited} (${(100 - successRate).toFixed(2)}%)`);
  console.log('');

  return {
    'stdout': textSummary(data, { indent: ' ', enableColors: true }),
    'summary.json': JSON.stringify(data, null, 2),
  };
}

function textSummary(data, options) {
  // Basic text summary for stdout
  return `
Load Test Results
=================

Total Duration: ${data.state.testRunDurationMs / 1000}s
Total Requests: ${data.metrics.http_reqs.values.count}
Request Rate: ${data.metrics.http_reqs.values.rate.toFixed(2)}/s

HTTP Status:
  200 OK: ${data.metrics.successful_requests.values.count}
  429 Rate Limited: ${data.metrics.rate_limit_errors.values.count}

Response Time:
  P95: ${data.metrics.response_time_p95.values['p(95)'].toFixed(2)}ms
  Average: ${data.metrics.http_req_duration.values.avg.toFixed(2)}ms
  Max: ${data.metrics.http_req_duration.values.max.toFixed(2)}ms

Rate Limit Headers Present: ${(data.metrics.rate_limit_header_present.values.rate * 100).toFixed(2)}%
`;
}
