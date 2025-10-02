APP=failover

build:
	cargo build --release

run:
	cargo run -- \
		--primary=https://example.com \
		--backup=https://example-backup.s3.amazonaws.com

docker:
	docker build -t failover:local .
	docker run --rm -p 8080:8080 \
	  -e SLACK_WEBHOOK_URL="" \
	  failover:local \
	  --primary=https://example.com \
	  --backup=https://example-backup.s3.amazonaws.com
