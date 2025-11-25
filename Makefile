.PHONY: help build test check fmt clippy pre-commit setup-hooks release-patch release-minor release-major release-dry-run cross-compile docker-build-linux

help:
	@echo "Scanner - Development & Release Management"
	@echo ""
	@echo "Development targets:"
	@echo "  build           - Build release binary"
	@echo "  test            - Run all tests"
	@echo "  check           - Check code compiles"
	@echo "  fmt             - Format code with rustfmt"
	@echo "  clippy          - Run clippy linter"
	@echo "  pre-commit      - Run all pre-commit checks"
	@echo "  setup-hooks     - Install pre-commit hooks"
	@echo ""
	@echo "Cross-compilation targets:"
	@echo "  cross-compile      - Build for all platforms (auto-uses Docker for Linux)"
	@echo "  docker-build-linux - Build Linux binaries using Docker"
	@echo ""
	@echo "Release targets:"
	@echo "  release-patch   - Create patch release with all binaries (0.1.0 → 0.1.1)"
	@echo "  release-minor   - Create minor release with all binaries (0.1.0 → 0.2.0)"
	@echo "  release-major   - Create major release with all binaries (0.1.0 → 1.0.0)"
	@echo "  release-dry-run - Test release process without changes"

build:
	cargo build --release

test:
	cargo test

check:
	cargo check --all-features --all-targets

fmt:
	cargo fmt --all

clippy:
	cargo clippy --all-features --all-targets -- -D warnings

pre-commit:
	@if [ -f .git/hooks/pre-commit ]; then \
		.git/hooks/pre-commit; \
	else \
		echo "Pre-commit hook not installed. Run: make setup-hooks"; \
		exit 1; \
	fi

setup-hooks:
	./setup-hooks.sh

release-patch:
	./release-all.sh --patch

release-minor:
	./release-all.sh --minor

release-major:
	./release-all.sh --major

release-dry-run:
	./release-all.sh --patch --dry-run

cross-compile:
	./cross-compile.sh

docker-build-linux:
	./docker-build-linux.sh
