.PHONY: help build check-release release-patch release-minor release-major release-dry-run

help:
	@echo "Scanner - Release Management"
	@echo ""
	@echo "Available targets:"
	@echo "  build           - Build release binary"
	@echo "  check-release   - Check if ready to release"
	@echo "  release-patch   - Create patch release (0.1.0 → 0.1.1)"
	@echo "  release-minor   - Create minor release (0.1.0 → 0.2.0)"
	@echo "  release-major   - Create major release (0.1.0 → 1.0.0)"
	@echo "  release-dry-run - Test release process without changes"

build:
	cargo build --release

check-release:
	./check-release.sh

release-patch:
	./release.sh --patch

release-minor:
	./release.sh --minor

release-major:
	./release.sh --major

release-dry-run:
	./release.sh --patch --dry-run
