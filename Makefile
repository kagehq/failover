APP=failover-proxy

build:
	cargo build --release

run:
	cargo run -- \
		--primary=https://example.com \
		--backup=https://example-backup.s3.amazonaws.com

docker:
	docker build -t failover-proxy:local .
	docker run --rm -p 8080:8080 \
	  -e SLACK_WEBHOOK_URL="" \
	  failover-proxy:local \
	  --primary=https://example.com \
	  --backup=https://example-backup.s3.amazonaws.com
