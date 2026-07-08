.PHONY: all build build-frontend build-backend build-release clean run test test-frontend test-backend install

PREFIX ?= $(HOME)/.cargo

all: build

build: build-backend build-frontend

build-release:
	cargo build --release
	cd frontend && nimble build -y -d:release

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

install: build-release
	install -d $(PREFIX)/bin
	install -m 755 target/release/maestro $(PREFIX)/bin/
	install -m 755 frontend/maestro_tui $(PREFIX)/bin/
