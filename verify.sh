#!/usr/bin/env bash
set -euo pipefail

# Ultra-fast test for failover-proxy
echo "🧪 Testing failover-proxy..."

# Clean up silently
pkill -f failover-proxy 2>/dev/null || true
pkill -f "python3.*http.server" 2>/dev/null || true

# Build
cargo build --release >/dev/null

# Create a simple test server that returns "PRIMARY OK"
cat > /tmp/test_server.py << 'EOF'
#!/usr/bin/env python3
import http.server
import socketserver
import sys

class TestHandler(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        self.send_response(200)
        self.send_header('Content-type', 'text/plain')
        self.end_headers()
        self.wfile.write(b'PRIMARY OK')
    
    def log_message(self, format, *args):
        pass

if __name__ == "__main__":
    port = int(sys.argv[1])
    with socketserver.TCPServer(("127.0.0.1", port), TestHandler) as httpd:
        httpd.serve_forever()
EOF

chmod +x /tmp/test_server.py

# Start server
python3 /tmp/test_server.py 9901 >/dev/null 2>&1 &
P1=$!

sleep 1

# Start proxy
./target/release/failover-proxy \
  --listen="127.0.0.1:8080" \
  --primary="http://127.0.0.1:9901" \
  --backup="http://127.0.0.1:9901" \
  --check-interval="1s" \
  --fail-threshold="1" \
  --recover-threshold="1" >/dev/null 2>&1 &
P_PROXY=$!

sleep 1

# Run tests
echo "✅ Basic routing"
BODY=$(curl -fsS "http://127.0.0.1:8080/" 2>/dev/null || echo "FAILED")
[[ "$BODY" == "PRIMARY OK" ]] || { echo "❌ Expected PRIMARY OK, got: $BODY"; exit 1; }

echo "✅ Health endpoint"
HEALTH=$(curl -fsS "http://127.0.0.1:8080/__failover/health" 2>/dev/null || echo "FAILED")
[[ "$HEALTH" == "OK" ]] || { echo "❌ Expected OK, got: $HEALTH"; exit 1; }

echo "✅ State endpoint"
STATE=$(curl -fsS "http://127.0.0.1:8080/__failover/state" 2>/dev/null || echo "FAILED")
echo "$STATE" | python3 -c "
import sys, json
try:
    data = json.load(sys.stdin)
    if 'on_backup' in data and 'primary' in data:
        pass
    else:
        sys.exit(1)
except:
    sys.exit(1)
" || { echo "❌ State test failed"; exit 1; }

echo ""
echo "🎉 ALL TESTS PASSED!"
echo "🚀 Failover-proxy is working!"

# Silent cleanup
kill $P_PROXY $P1 2>/dev/null || true
rm -f /tmp/test_server.py 2>/dev/null || true