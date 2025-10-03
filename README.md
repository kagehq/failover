# Failover

**Zero-downtime proxy with instant Slack/Discord alerts. Setup in 30 seconds.**

A tiny reverse proxy that sits in front of your app and makes downtime invisible. Routes traffic to your primary service, fails over instantly to backup when things break, auto-recovers, and posts incident reports to Slack/Discord automatically.

## Quick Start

```bash
# One-line installer
curl -L https://raw.githubusercontent.com/kagehq/failover/main/install.sh | bash

# Run it
./failover \
  --primary=https://yourapp.com \
  --backup=https://backup.s3.amazonaws.com \
  --webhook-url=https://hooks.slack.com/YOUR/WEBHOOK
```

**Or deploy instantly:**
- [Deploy to Render](https://render.com/deploy) • [Railway](deploy/railway.toml) • [Fly.io](deploy/fly.toml) • [See all options →](deploy/DEPLOY.md)

## Why Failover?

**Traditional HA is broken:**
- ❌ Costs $50K+ (infrastructure duplication)
- ❌ Takes weeks to setup (cluster configs)
- ❌ Causes flapping (primary↔backup loops)
- ❌ Loses data (replication lag)

**Failover is different:**
- ✅ $0 cost (uses existing backup infrastructure)
- ✅ 30-second setup (one command)
- ✅ No flapping (intelligent health checks)
- ✅ No data loss (stateless proxy)

[Read detailed comparison →](DETAILED.md#comparison)

## What Engineers Are Saying

> "Scaling to distributed failover creates a 'death valley' of reliability due to config errors and interconnect failures."  

> "Active-active setups risk race conditions while active-passive causes data loss from replication lag."  

[Read more industry feedback →](DETAILED.md#what-engineers-are-saying)

## Core Features

- **Auto Failover** - Sub-second switching when primary fails
- **Auto Recovery** - Switches back when primary is healthy  
- **Slack/Discord Alerts** - Instant incident reports with timeline and duration
- **Health Monitoring** - Continuous checks with configurable thresholds
- **Zero Config** - Works with any HTTP backend (app, S3, CDN, static site)

## Is This For You?

**✅ Perfect if:**
- You're a startup/indie hacker without $50K for HA infrastructure
- Your users can tolerate brief read-only mode during outages
- You need HA NOW without weeks of setup

**❌ Not ideal if:**
- You're a bank with 99.999% compliance requirements
- You need real-time writes during any failure scenario
- You have budget for enterprise HA solutions

**The reality:** 80% of web apps don't need enterprise HA. They need solid, affordable failover. [Read the honest trade-offs →](DETAILED.md#trade-offs)

## Documentation

- 📖 [Detailed Guide](DETAILED.md) - In-depth explanation, comparisons, expert quotes
- 🚀 [Deployment Guide](deploy/DEPLOY.md) - All deployment platforms and options
- ⚙️ [Configuration](DETAILED.md#configuration) - CLI flags, YAML config, environment variables
- 📊 [Monitoring](DETAILED.md#monitoring) - Health endpoints, incident reports, webhooks
- 🧪 [Testing](DETAILED.md#testing) - How to test failover behavior

## Community & Support

[![Discord](https://img.shields.io/badge/Discord-Join%20our%20community-7289DA?style=for-the-badge&logo=discord&logoColor=white)](https://discord.gg/KqdBcqRk5E)

## License

This project is licensed under the FSL-1.1-MIT License. See the LICENSE file for details.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request
