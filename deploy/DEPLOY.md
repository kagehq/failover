# One-Click Deploy Guide ✅

Deploy Failover to your favorite cloud platform in minutes.

> **Note:** All deployment configuration files are in this `deploy/` directory. The quick-deploy script will automatically handle paths for you.

## Railway (Recommended) 🚂

[![Deploy on Railway](https://railway.app/button.svg)](https://railway.app/template/failover)

**Steps:**
1. Click the deploy button above
2. Set environment variables:
   - `PRIMARY_URL` - Your primary service URL
   - `BACKUP_URL` - Your backup service URL
   - `WEBHOOK_URL` (optional) - Slack/Discord webhook
3. Deploy! Railway will automatically:
   - Build from Dockerfile
   - Assign a public URL
   - Auto-restart on failures

**Configuration:**
Railway automatically uses `deploy/railway.json` for Docker builds or `deploy/railway.toml` for Nixpacks.

---

## Fly.io 🚀

```bash
# Install flyctl
curl -L https://fly.io/install.sh | sh

# Login
flyctl auth login

# Deploy (from project directory)
flyctl launch
flyctl secrets set PRIMARY_URL=https://yourapp.com
flyctl secrets set BACKUP_URL=https://backup.s3.amazonaws.com
flyctl secrets set WEBHOOK_URL=https://hooks.slack.com/services/YOUR/WEBHOOK
flyctl deploy
```

Fly.io will use the included `deploy/fly.toml` configuration.

---

## Render 🎨

[![Deploy to Render](https://render.com/images/deploy-to-render-button.svg)](https://render.com/deploy)

**Steps:**
1. Fork this repository
2. Create a new Web Service on Render
3. Connect your forked repository
4. Render will auto-detect `deploy/render.yaml`
5. Set environment variables in the dashboard
6. Deploy!

**Environment Variables:**
- `PRIMARY_URL` (required)
- `BACKUP_URL` (required)
- `WEBHOOK_URL` (optional)
- `WEBHOOK_FORMAT` (optional, default: slack)

---

## Vercel ⚡

> **Note:** Vercel is optimized for serverless/frontend apps. For a long-running reverse proxy, Railway or Fly.io are better choices.

If you still want to use Vercel:

```bash
# Install Vercel CLI
npm i -g vercel

# Deploy
vercel --build-env PRIMARY_URL=https://yourapp.com \
       --build-env BACKUP_URL=https://backup.s3.amazonaws.com

# Set production environment variables
vercel env add PRIMARY_URL
vercel env add BACKUP_URL
vercel env add WEBHOOK_URL

# Redeploy
vercel --prod
```

---

## Cloudflare 🟠

> **Note:** Cloudflare Workers don't support long-running TCP proxies. Best approach: Deploy to Railway/Fly.io and use Cloudflare as a CDN in front.

**Recommended Setup:**
1. Deploy Failover to Railway/Fly.io (see above)
2. Get your deployment URL (e.g., `failover.railway.app`)
3. In Cloudflare:
   - Add your domain
   - Create a CNAME record pointing to your Railway/Fly URL
   - Enable Cloudflare Proxy (orange cloud)
4. Now you have: `User → Cloudflare CDN → Failover Proxy → Your App`

This gives you:
- Cloudflare's DDoS protection
- Global CDN
- Automatic failover via Failover Proxy
- SSL/TLS encryption

**Alternative: Cloudflare Workers (Limited)**
If you need edge computing, you can use Cloudflare Workers as a simple router:

```javascript
// worker.js - Simple routing to Failover
export default {
  async fetch(request) {
    const failoverUrl = 'https://failover.railway.app';
    return fetch(failoverUrl + new URL(request.url).pathname, request);
  }
}
```

But this adds an extra hop and doesn't provide the same failover capabilities.

---

## Docker Compose (Self-Hosted) 🐳

```bash
# Clone the repo
git clone https://github.com/kagehq/failover.git
cd failover

# Create .env file
cat > .env << EOF
PRIMARY_URL=https://yourapp.com
BACKUP_URL=https://backup.s3.amazonaws.com
WEBHOOK_URL=https://hooks.slack.com/services/YOUR/WEBHOOK
PORT=8080
EOF

# Deploy
docker-compose up -d
```

Create `docker-compose.yml`:
```yaml
version: '3.8'
services:
  failover:
    build: .
    ports:
      - "8080:8080"
    environment:
      - PRIMARY_URL=${PRIMARY_URL}
      - BACKUP_URL=${BACKUP_URL}
      - WEBHOOK_URL=${WEBHOOK_URL}
      - WEBHOOK_FORMAT=slack
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/__failover/health"]
      interval: 30s
      timeout: 10s
      retries: 3
```

---

## Environment Variables Reference

All platforms support these environment variables:

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `PRIMARY_URL` | ✅ Yes | - | Primary service URL |
| `BACKUP_URL` | ✅ Yes | - | Backup service URL |
| `PORT` | No | `8080` | Port to listen on |
| `CHECK_INTERVAL` | No | `2s` | Health check interval |
| `FAIL_THRESHOLD` | No | `3` | Consecutive failures before failover |
| `RECOVER_THRESHOLD` | No | `2` | Consecutive successes before recovery |
| `WEBHOOK_URL` | No | - | Slack/Discord webhook for notifications |
| `WEBHOOK_FORMAT` | No | `slack` | Webhook format (`slack` or `discord`) |
| `MAX_BODY` | No | `10MB` | Max request body size |
| `JSON_LOGS` | No | `false` | Enable JSON logging |

---

## Quick Deploy Script

We provide an interactive deployment script in this directory:

```bash
cd deploy
chmod +x quick-deploy.sh
./quick-deploy.sh
```

The script will:
- Guide you through platform selection
- Collect your configuration
- Automatically deploy to your chosen platform
- Handle all the setup for you

---

## Post-Deployment

After deploying to any platform:

1. **Get your deployment URL** (e.g., `https://failover-abc123.railway.app`)

2. **Update your DNS:**
   - Point your domain to the deployment URL
   - Or use it directly for testing

3. **Verify it's working:**
   ```bash
   curl https://your-deployment-url/__failover/health
   # Should return: OK
   
   curl https://your-deployment-url/__failover/state
   # Should return JSON with status
   ```

4. **Monitor:**
   - Check your Slack/Discord for failover notifications
   - Use `/__failover/state` endpoint for monitoring
   - Set up your existing monitoring tools to track the proxy


