.PHONY: build
build: build/frontend build/api

.PHONY: build/frontend
build/frontend:
	cd ./frontend/ && yarn build

.PHONY: build/api
build/api:
	cargo build

