#!/bin/bash

# CORS Testing Script for API Gateway
# This script demonstrates CORS functionality using curl

SERVER_URL="http://localhost:3000"
echo "🌐 Testing CORS for API Gateway at $SERVER_URL"
echo "=================================================="

# Test 1: Basic GET request
echo ""
echo "1️⃣ Testing Basic GET Request"
echo "-----------------------------"
curl -v -X GET "$SERVER_URL/" \
  -H "Origin: http://localhost:8080" \
  -H "Content-Type: application/json" \
  2>&1 | grep -E "(< HTTP|< Access-Control|> Origin|> Content-Type)"

# Test 2: POST request with custom headers
echo ""
echo "2️⃣ Testing POST Request with Headers"
echo "------------------------------------"
curl -v -X POST "$SERVER_URL/" \
  -H "Origin: https://example.com" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer test-token" \
  -H "X-Request-Id: curl-test-$(date +%s)" \
  -d '{"test": "data", "timestamp": "'$(date -Iseconds)'"}' \
  2>&1 | grep -E "(< HTTP|< Access-Control|> Origin|> Content-Type|> Authorization|> X-Request-Id)"

# Test 3: OPTIONS preflight request
echo ""
echo "3️⃣ Testing OPTIONS Preflight Request"
echo "-------------------------------------"
curl -v -X OPTIONS "$SERVER_URL/" \
  -H "Origin: http://localhost:3000" \
  -H "Access-Control-Request-Method: POST" \
  -H "Access-Control-Request-Headers: Content-Type, Authorization, X-Request-Id" \
  2>&1 | grep -E "(< HTTP|< Access-Control|> Origin|> Access-Control-Request)"

# Test 4: Health check endpoint
echo ""
echo "4️⃣ Testing Health Check Endpoint"
echo "---------------------------------"
curl -v -X GET "$SERVER_URL/healthz" \
  -H "Origin: https://staging.example.com" \
  -H "X-Request-Id: health-check-$(date +%s)" \
  2>&1 | grep -E "(< HTTP|< Access-Control|> Origin|> X-Request-Id)"

# Test 5: Test with different origins
echo ""
echo "5️⃣ Testing Different Origins"
echo "-----------------------------"
for origin in "http://localhost:8080" "https://app.example.com" "https://staging.example.com"; do
  echo "Testing origin: $origin"
  curl -s -X GET "$SERVER_URL/" \
    -H "Origin: $origin" \
    -H "X-Request-Id: origin-test-$(date +%s)" \
    -w "Status: %{http_code}, CORS-Origin: %{header_json[Access-Control-Allow-Origin]}\n" \
    -o /dev/null
done

echo ""
echo "✅ CORS Testing Complete!"
echo ""
echo "What to look for:"
echo "- Access-Control-Allow-Origin: Should match the Origin header or be '*'"
echo "- Access-Control-Allow-Methods: Should include GET, POST, OPTIONS"
echo "- Access-Control-Allow-Headers: Should include Content-Type, Authorization, X-Request-Id"
echo "- Access-Control-Expose-Headers: Should include X-Request-Id"
echo "- HTTP 200 status codes for successful requests"
