.PHONY: all build build-frontend build-backend clean run test test-frontend test-backend

all: build

build: build-backend build-frontend

build-frontend:
	cd frontend && nimble build -y

build-backend:
	cargo build

clean:
	cargo clean
	rm -rf frontend/nimcache frontend/maestro_tui

run: build
	MAESTRO_TUI="$(PWD)/frontend/maestro_tui" cargo run -- $(ARGS)

test: test-backend test-frontend

test-backend:
	cargo test

test-frontend:
	cd frontend && nimble test
