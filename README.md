# Failover

A tiny reverse proxy that sits in front of your app or API and makes downtime invisible. It routes all traffic to your primary service, fails over instantly to a backup when things break, and then fails back automatically when your primary recovers. No SDKs, no rewrites, just point DNS and sleep better.

## Why This Exists

Downtime isn’t just an inconvenience — it destroys trust, revenue, and SLAs. Even the biggest companies suffer:
- 2024 – CrowdStrike update outage → airports, banks, and hospitals went offline worldwide.
- 2023 – Cloudflare outage → thousands of sites (including Shopify stores) disappeared for hours.
- 2021 – Facebook global outage → 3 billion users saw “site not available.”
-	AWS/GCP/Azure regularly have region-level incidents that take apps offline for hours.

If it happens to them, it can happen to you.

## What Failover Does
- Routes traffic to your primary service.
- Continuously health-checks it.
- If it fails → instantly routes traffic to your backup (S3, static copy, or secondary region).
- Automatically recovers back when primary is healthy.
- Exposes /__failover/health and /__failover/state for monitoring, and can notify you (e.g., Slack) when failovers happen.

Your users keep seeing your site. You keep your SLA. Downtime becomes invisible.

## Quick Start

### Download Pre-built Binary

Download the latest release for your platform:

```bash
# Linux x86_64
curl -L https://github.com/kagehq/failover/releases/latest/download/failover-linux-x86_64 -o failover
chmod +x failover

# Linux ARM64
curl -L https://github.com/kagehq/failover/releases/latest/download/failover-linux-aarch64 -o failover
chmod +x failover

# macOS x86_64 (Intel)
curl -L https://github.com/kagehq/failover/releases/latest/download/failover-macos-x86_64 -o failover
chmod +x failover

# macOS ARM64 (Apple Silicon)
curl -L https://github.com/kagehq/failover/releases/latest/download/failover-macos-aarch64 -o failover
chmod +x failover

# Windows x86_64
curl -L https://github.com/kagehq/failover/releases/latest/download/failover-windows-x86_64.exe -o failover.exe
```

Then run:
```bash
./failover --primary=https://myapp.com --backup=https://myapp-backup.s3.amazonaws.com
```

### Run from source:

```bash
cargo run --release -- \
  --primary=https://myapp.com \
  --backup=https://myapp-backup.s3.amazonaws.com
```

### What this does:
- Compiles Failover Proxy (release mode for speed).
- Starts listening on `0.0.0.0:8080`.
- Routes all traffic to `https://myapp.com`.
- If primary fails → instantly switches to `https://myapp-backup.s3.amazonaws.com`.
- Auto-recovers back to primary when healthy.


### Run from Docker:

#### 1. Download or Run

Build and run the Docker container with instant failover from `primary` to `backup`:

```bash
# Build the Docker image
docker build -t failover:latest .

# Run the proxy
docker run --rm -p 8080:8080 failover:latest \
  --primary=https://myapp.com --backup=https://myapp-backup.s3.amazonaws.com
```
That's it, no config files, no setup. The container runs, and traffic is proxied with health-check failover baked in.  

#### 2. Configure
- Primary = their live service (app, API, site).
-  Backup = static copy, alternate region, cached mirror, even a “sorry page.”
- Configured via flags or a YAML file.

#### 3. Point DNS
- They point myapp.com (or a subdomain like app.mycompany.com) to the proxy.
- From then on, all requests hit Failover Proxy first.

#### 4. Automatic Failover
- While healthy: traffic → primary.
- If health checks fail 3x in a row (configurable): traffic → backup.
- While failing: proxy keeps checking primary in the background.
- After 2x successful health checks: traffic → primary again.

#### 5. Observe
- Check `http://proxy:8080/__failover/state` to see:

```json
  { "on_backup": true, "since_unix": 1738512461, "primary": "...", "backup": "..." }
```

## ASCII Diagram

BEFORE Failover Proxy
------------------------------
 User  →  DNS  →  Primary App
                   (Down = ❌ downtime)

AFTER Failover Proxy
------------------------------
 User  →  DNS  →  Failover Proxy
                        │
         ┌──────────────┴───────────────┐
         ▼                              ▼
   Primary App (healthy ✅)     Backup (S3/CloudFront, etc.)
         (Down = traffic auto-fails here ✅)

## Configuration

All options can be set via command-line flags or a YAML config file:

### Command Line Options

```bash
failover --help
```

**Required:**
- `--primary <URL>` - Primary upstream URL
- `--backup <URL>` - Backup upstream URL

**Optional:**
- `--listen <ADDRESS>` - Listen address (default: `0.0.0.0:8080`)
- `--check-interval <DURATION>` - Health check interval (default: `2s`)
- `--fail-threshold <COUNT>` - Fail threshold (default: `3`)
- `--recover-threshold <COUNT>` - Recover threshold (default: `2`)
- `--max-body <SIZE>` - Max request body size (default: `10MB`)
- `--config <FILE>` - Config file path
- `--json-logs` - Enable JSON logging

### Example Config File
Create `config.yaml`:

```yaml
listen: "0.0.0.0:8080"
primary: "https://myapp.com"
backup: "https://myapp-backup.s3.amazonaws.com"
check_interval: "2s"
fail_threshold: 3
recover_threshold: 2
max_body: "10MB"
json_logs: false
```

Then run: `failover --config config.yaml`

## Monitoring Endpoints

- `GET /__failover/health` - Simple health check (returns 200 OK)
- `GET /__failover/state` - Detailed status and configuration

## Testing
Run the comprehensive test suite:

```bash
./verify.sh
```

This tests:
- **Basic Functionality**: Primary routing, automatic failover, recovery
- **Health Endpoints**: Health and state endpoint validation
- **HTTP Methods**: GET, POST, PUT, DELETE request handling
- **Request Processing**: Body handling and header forwarding
- **Configuration**: JSON logging and config file loading
- **Production Readiness**: All edge cases and error scenarios


## License

This project is licensed under the FSL-1.1-MIT License. See the LICENSE file for details.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request
