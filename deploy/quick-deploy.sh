#!/bin/bash
set -e

# Navigate to project root (parent directory)
cd "$(dirname "$0")/.."

echo "🚀 Failover One-Click Deploy"
echo "=============================="
echo ""

# Check if required commands exist
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Get platform choice
echo "Choose deployment platform:"
echo "1) Railway (Recommended)"
echo "2) Fly.io"
echo "3) Render"
echo "4) Docker (Self-hosted)"
echo ""
read -p "Enter choice (1-4): " PLATFORM

# Get configuration
echo ""
echo "Configuration:"
read -p "Primary URL: " PRIMARY
read -p "Backup URL: " BACKUP
read -p "Webhook URL (optional, press Enter to skip): " WEBHOOK

case $PLATFORM in
  1)
    echo ""
    echo "Deploying to Railway..."
    
    if ! command_exists railway; then
        echo "❌ Railway CLI not found. Installing..."
        npm install -g @railway/cli || {
            echo "❌ Failed to install Railway CLI. Please install Node.js first."
            exit 1
        }
    fi
    
    railway login
    railway up
    railway variables set PRIMARY_URL="$PRIMARY" BACKUP_URL="$BACKUP"
    [ -n "$WEBHOOK" ] && railway variables set WEBHOOK_URL="$WEBHOOK"
    
    echo ""
    echo "✅ Deployed to Railway!"
    echo "Opening dashboard..."
    railway open
    ;;
    
  2)
    echo ""
    echo "Deploying to Fly.io..."
    
    if ! command_exists flyctl; then
        echo "❌ Fly CLI not found. Installing..."
        curl -L https://fly.io/install.sh | sh
        echo "Please restart your shell and run this script again."
        exit 1
    fi
    
    flyctl auth login
    flyctl launch --no-deploy --config deploy/fly.toml
    flyctl secrets set PRIMARY_URL="$PRIMARY" BACKUP_URL="$BACKUP"
    [ -n "$WEBHOOK" ] && flyctl secrets set WEBHOOK_URL="$WEBHOOK"
    flyctl deploy
    
    echo ""
    echo "✅ Deployed to Fly.io!"
    echo "Opening app..."
    flyctl open
    ;;
    
  3)
    echo ""
    echo "Deploying to Render..."
    echo ""
    echo "📝 Manual steps required:"
    echo "1. Fork this repository on GitHub"
    echo "2. Visit: https://render.com/deploy"
    echo "3. Connect your forked repository"
    echo "4. Set environment variables:"
    echo "   - PRIMARY_URL=$PRIMARY"
    echo "   - BACKUP_URL=$BACKUP"
    [ -n "$WEBHOOK" ] && echo "   - WEBHOOK_URL=$WEBHOOK"
    echo ""
    echo "Opening Render..."
    open "https://render.com/deploy" 2>/dev/null || xdg-open "https://render.com/deploy" 2>/dev/null || echo "Please visit: https://render.com/deploy"
    ;;
    
  4)
    echo ""
    echo "Deploying with Docker..."
    
    if ! command_exists docker; then
        echo "❌ Docker not found. Please install Docker first."
        echo "Visit: https://docs.docker.com/get-docker/"
        exit 1
    fi
    
    # Create .env file
    cat > .env << EOF
PRIMARY_URL=$PRIMARY
BACKUP_URL=$BACKUP
${WEBHOOK:+WEBHOOK_URL=$WEBHOOK}
PORT=8080
EOF
    
    echo "Building Docker image..."
    docker build -t failover:latest .
    
    echo "Starting container..."
    docker run -d \
      --name failover \
      -p 8080:8080 \
      --env-file .env \
      --restart unless-stopped \
      failover:latest \
      --primary="$PRIMARY" \
      --backup="$BACKUP" \
      ${WEBHOOK:+--webhook-url="$WEBHOOK"}
    
    echo ""
    echo "✅ Deployed locally!"
    echo ""
    echo "Service running at: http://localhost:8080"
    echo "Health check: http://localhost:8080/__failover/health"
    echo "Status: http://localhost:8080/__failover/state"
    echo ""
    echo "To view logs: docker logs -f failover"
    echo "To stop: docker stop failover"
    echo "To remove: docker rm -f failover"
    ;;
    
  *)
    echo "❌ Invalid choice. Please run the script again."
    exit 1
    ;;
esac

echo ""
echo "🎉 Deployment complete!"

