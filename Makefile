.PHONY: build
build: build/frontend build/api

.PHONY: build/frontend
build/frontend:
	cd ./frontend/ && yarn build

.PHONY: build/api
build/api:
	cargo build

.PHONY: lint
lint:
	cargo fmt --check
	cargo clippy
	cargo machete

.PHONY: test
test:
	cargo llvm-cov --no-fail-fast

.PHONY: test-ci
test-ci:
	cargo llvm-cov --no-fail-fast

