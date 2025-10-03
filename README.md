# Failover

A tiny reverse proxy that sits in front of your app or API and makes downtime invisible. Routes traffic to your primary service, fails over instantly to backup when things break, auto-recovers, and posts incident reports to Slack/Discord automatically.

## What Failover Does

- Routes traffic to your primary service.
- Continuously health-checks it.
- If it fails → instantly routes traffic to your backup (S3, static copy, or secondary region).
- Automatically recovers back when primary is healthy.
- Get Slack/Discord notifications with detailed timelines when failover happens. Zero effort post-mortems.

Your users keep seeing your site. You keep your SLA. Downtime becomes invisible.

## Why This Exists

Downtime isn’t just an inconvenience — it destroys trust, revenue, and SLAs. Even the biggest companies suffer:
- 2024 – CrowdStrike update outage → airports, banks, and hospitals went offline worldwide.
- 2023 – Cloudflare outage → thousands of sites (including Shopify stores) disappeared for hours.
- 2021 – Facebook global outage → 3 billion users saw “site not available.”
-	AWS/GCP/Azure regularly have region-level incidents that take apps offline for hours.

If it happens to them, it can happen to you.

## The Problem with Traditional Failover

Enterprise failover systems have critical flaws that cause more problems than they solve:

**❌ Expensive** - Triple your infrastructure costs for redundancy, testing, and maintenance  
**❌ Complex Setup** - Active-active/active-passive configs lead to race conditions and misconfigurations  
**❌ Downtime During Switchover** - "Flapping" between primary/backup causes repeated outages  
**❌ Data Loss Risk** - Replication lag means lost writes during failure  
**❌ Network Dependencies** - Latency, DNS issues, and misconfigs break handoffs  
**❌ Hardware Failures** - BSODs, VM freezes, resource contention brick entire clusters  

**Failover solves this differently:**

✅ **Zero Infrastructure Duplication** - Use existing backup (S3, CDN, static site, secondary region)  
✅ **Sub-Second Switchover** - No flapping, instant routing change at the proxy layer  
✅ **No Data Loss Risk** - Stateless proxy, no replication lag to worry about  
✅ **Simple Setup** - One command, no cluster configs or coordination required  
✅ **Works with Any Backend** - Primary = your app, Backup = literally anything that serves HTTP  

**Cost comparison:**  
Traditional failover: $50K+ for complex HA clusters + ongoing maintenance  
Failover: $0 (open source) + uses infrastructure you already have

## Failover vs Traditional HA Solutions

| Feature | AWS Route53 Failover | Kubernetes HA | **Failover** |
|---------|---------------------|---------------|--------------|
| **Setup Time** | Hours-Days | Days-Weeks | **30 seconds** |
| **Monthly Cost** | $50-500 | $500-5K | **$0 (OSS)** |
| **Infrastructure Duplication** | Required | Required | **Not Required** |
| **Switchover Time** | 60-180 seconds | 30-90 seconds | **< 1 second** |
| **Data Loss Risk** | Replication lag | Replication lag | **Zero (stateless)** |
| **Complexity** | High | Very High | **One command** |
| **Flapping Issues** | Common | Common | **Eliminated** |
| **Incident Reports** | Manual | Manual | **Auto to Slack/Discord** |

## What Engineers Are Saying About Traditional Failover

Real problems from the field:

> "Active-active setups risk race conditions and complexity, while active-passive can lead to downtime and data loss if replication lags. Teams overestimate availability without understanding these trade-offs." 

> "Scaling from single-machine to distributed setups spikes outage risks due to config errors and interconnect failures—a 'death valley' in reliability."

> "If a failover playbook fails midway, you're left in a hybrid state (some updated, some not), birthing outages from config drift."

> "Multiple simultaneous node failures at scale demand careful replication—otherwise quorum latency or saturation hits hard." 

**The common theme:** Traditional failover adds complexity that creates NEW failure modes.

**Failover's approach:** Stateless proxy = no replication, no config drift, no race conditions. Just instant routing.

## What Failover Solves (And What It Doesn't)

**✅ Problems Failover SOLVES:**
- **Race Conditions** - Stateless proxy eliminates active-active coordination complexity
- **Config Drift** - Single binary, no distributed state to get out of sync
- **Replication Lag Data Loss** - No replication needed at proxy layer = zero data loss risk
- **Infrastructure Costs** - Use existing backups (S3, CDN) instead of duplicating everything
- **Complex Setup** - One command vs. weeks of cluster configuration
- **Flapping** - Intelligent health checks prevent primary↔backup toggling

**❌ Problems Failover DOESN'T Solve:**
- **Database-Level HA** - You still need proper DB clustering for stateful data
- **Real-Time Writes During Failover** - Backup serves read-only/cached content
- **Sub-10ms Failover** - Proxy adds ~1s switchover (vs. enterprise active-active)
- **Multi-Region Active-Active** - Single proxy point, not geo-distributed writes
- **Compliance Requirements** - If you need 99.999% SLA guarantees, use enterprise HA

**The Honest Trade-off:**

| Aspect | Enterprise HA | Failover |
|--------|---------------|----------|
| **Handles ALL scenarios** | ✅ Yes | ❌ No (80% of cases) |
| **Cost** | $50K+/year | $0 (OSS) |
| **Complexity** | Very High | One command |
| **Setup Time** | Weeks | 30 seconds |
| **Good Enough For** | Banks, stock exchanges | Startups, SaaS, APIs, web apps |

**Choose Failover if:**
- You're a startup/indie hacker without $50K+ for HA infrastructure
- Your users can tolerate brief read-only mode during primary outages
- You need HA NOW without weeks of setup
- "Pretty good" uptime is acceptable (vs. "absolutely perfect")

**Choose Enterprise HA if:**
- You're a bank, stock exchange, or medical device company
- You have compliance requirements for 99.999% uptime
- You absolutely need real-time writes during any failure scenario
- You have the budget and team to manage complex distributed systems

**The reality:** Most web apps, APIs, and SaaS products don't need enterprise HA. They need "good enough" HA at a price they can afford. That's Failover.

## Community & Support

Join our Discord community for discussions, support, and updates:

[![Discord](https://img.shields.io/badge/Discord-Join%20our%20community-7289DA?style=for-the-badge&logo=discord&logoColor=white)](https://discord.gg/KqdBcqRk5E)


## Quick Start

### Download Pre-built Binary

**One-line installer (auto-detects your platform):**

```bash
curl -L https://raw.githubusercontent.com/kagehq/failover/main/install.sh | bash
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

## Auto Incident Reports

Failover automatically sends detailed incident reports when failover events occur. Get notified instantly on Slack, Discord, or any webhook-compatible service when your primary service fails or recovers.

### Quick Setup

**Slack:**
```bash
failover \
  --primary=https://myapp.com \
  --backup=https://myapp-backup.s3.amazonaws.com \
  --webhook-url=https://hooks.slack.com/services/YOUR/WEBHOOK/URL \
  --webhook-format=slack
```

**Discord:**
```bash
failover \
  --primary=https://myapp.com \
  --backup=https://myapp-backup.s3.amazonaws.com \
  --webhook-url=https://discord.com/api/webhooks/YOUR/WEBHOOK/URL \
  --webhook-format=discord
```

**Or via config file:**
```yaml
webhook_url: "https://hooks.slack.com/services/YOUR/WEBHOOK/URL"
webhook_format: "slack"  # or "discord"
```

### Notification Example

When failover occurs, you'll receive a notification like:
```
🚨 Failover Incident Report
Event: FAILOVER
Timestamp: 2025-10-03T10:30:45Z
Primary: https://myapp.com
Backup: https://myapp-backup.s3.amazonaws.com
Details: Primary service failed after 3 consecutive health check failures.
```

When primary recovers:
```
✅ Failover Incident Report
Event: RECOVERY
Timestamp: 2025-10-03T10:45:30Z
Duration: 894 seconds
Details: Primary service has recovered and is now healthy. Traffic restored to primary.
```

### Setting Up Webhooks

**Slack:**
1. Go to https://api.slack.com/messaging/webhooks
2. Create a new webhook for your workspace
3. Copy the webhook URL
4. Use with `--webhook-url` flag

**Discord:**
1. Go to Server Settings → Integrations → Webhooks
2. Create a new webhook
3. Copy the webhook URL
4. Use with `--webhook-url` and `--webhook-format=discord`


## One-Click Deploy

Deploy Failover to production in minutes with zero configuration hassle.

### Railway (Recommended)
[![Deploy on Railway](https://railway.app/button.svg)](https://railway.app/template/failover)

Click the button above, set your `PRIMARY_URL` and `BACKUP_URL` environment variables, and deploy instantly.

**See [deploy/DEPLOY.md](deploy/DEPLOY.md) for complete deployment guides and more options**

### Quick Deploy Script
The script will guide you through deploying to your chosen platform with automatic configuration.

```bash
# Interactive deployment wizard
cd deploy
chmod +x quick-deploy.sh
./quick-deploy.sh
```

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
- `--webhook-url <URL>` - Webhook URL for incident notifications (Slack, Discord, etc.)
- `--webhook-format <FORMAT>` - Webhook format: `slack` or `discord` (default: `slack`)

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

# Auto Incident Reports (optional)
webhook_url: "https://hooks.slack.com/services/YOUR/WEBHOOK/URL"
webhook_format: "slack"  # or "discord"
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


## License

This project is licensed under the FSL-1.1-MIT License. See the LICENSE file for details.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request
