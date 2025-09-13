# 🌐 CORS Demo Guide for API Gateway

This guide shows you how to test and demonstrate the CORS functionality we just implemented.

## 🚀 Quick Start Demo

### 1. Start the Server
```bash
# Terminal 1: Start the API Gateway
cd /home/shawnc/Documents/public-video-service
cargo run --package api-gateway --bin api-gateway
```

The server will start on `http://localhost:3000` with default CORS settings (`cors_origins: ["*"]`).

### 2. Test with Browser (Easiest Demo)
```bash
# Open the test page in your browser
open /home/shawnc/Documents/public-video-service/cors-test.html
# OR
firefox /home/shawnc/Documents/public-video-service/cors-test.html
```

The HTML page will:
- ✅ Make cross-origin requests to your API Gateway
- ✅ Test GET, POST, and OPTIONS methods
- ✅ Test all allowed headers (Content-Type, Authorization, X-Request-Id)
- ✅ Show CORS headers in the response
- ✅ Demonstrate X-Request-Id exposure

### 3. Test with Command Line
```bash
# Run the automated curl tests
./test-cors.sh
```

This will test:
- ✅ Basic GET requests
- ✅ POST requests with headers
- ✅ OPTIONS preflight requests
- ✅ Different origins
- ✅ Health check endpoint

## 🎯 What to Look For

### Successful CORS Response Headers:
```
Access-Control-Allow-Origin: *                    # (or specific origin)
Access-Control-Allow-Methods: GET, POST, OPTIONS
Access-Control-Allow-Headers: Content-Type, Authorization, X-Request-Id
Access-Control-Expose-Headers: X-Request-Id
```

### Test Different Configurations:

#### Test 1: Wildcard Origins (Default)
```bash
# Server already running with cors_origins: ["*"]
curl -v -H "Origin: https://any-domain.com" http://localhost:3000/
# Should work - allows any origin
```

#### Test 2: Specific Origins
```bash
# Stop server (Ctrl+C), then restart with specific origins
APP_CORS_ORIGINS='["http://localhost:8080", "https://app.example.com"]' cargo run --package api-gateway --bin api-gateway
```

Then test:
```bash
# This should work (allowed origin)
curl -v -H "Origin: http://localhost:8080" http://localhost:3000/

# This should fail (not allowed origin)
curl -v -H "Origin: https://evil.com" http://localhost:3000/
```

## 🎬 Demo Script

### For Live Demo:
1. **Start server**: `cargo run --package api-gateway --bin api-gateway`
2. **Show browser test**: Open `cors-test.html` and click buttons
3. **Show curl test**: Run `./test-cors.sh`
4. **Show config**: Point to `config.dev.toml` CORS section
5. **Show validation**: Try invalid origin in config

### Key Points to Highlight:
- ✅ **Security**: Wildcard vs specific origins
- ✅ **Headers**: Content-Type, Authorization, X-Request-Id
- ✅ **Methods**: GET, POST, OPTIONS (preflight)
- ✅ **Validation**: Invalid origins rejected at startup
- ✅ **Exposure**: X-Request-Id available to browsers

## 🔧 Troubleshooting

### Server Won't Start:
```bash
# Check if port is in use
lsof -i :3000

# Try different port
APP_PORT=3001 cargo run --package api-gateway --bin api-gateway
```

### CORS Not Working:
```bash
# Check server logs for CORS configuration
RUST_LOG=info cargo run --package api-gateway --bin api-gateway

# Verify config loading
grep -A 5 "CORS origins" target/debug/api-gateway.log
```

### Browser Blocked:
- Check browser console for CORS errors
- Verify server is running on correct port
- Try different browser or incognito mode

## 📊 Expected Results

### ✅ Success Indicators:
- HTTP 200 responses
- CORS headers present
- No browser console errors
- X-Request-Id visible in responses

### ❌ Failure Indicators:
- HTTP 403/404 responses
- Missing CORS headers
- Browser CORS errors
- "Access to fetch at ... has been blocked by CORS policy"
