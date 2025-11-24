.PHONY: help build release-patch release-minor release-major release-dry-run

help:
	@echo "Scanner - Release Management"
	@echo ""
	@echo "Available targets:"
	@echo "  build           - Build release binary"
	@echo "  release-patch   - Create patch release with all binaries (0.1.0 → 0.1.1)"
	@echo "  release-minor   - Create minor release with all binaries (0.1.0 → 0.2.0)"
	@echo "  release-major   - Create major release with all binaries (0.1.0 → 1.0.0)"
	@echo "  release-dry-run - Test release process without changes"

build:
	cargo build --release

release-patch:
	./release-all.sh --patch

release-minor:
	./release-all.sh --minor

release-major:
	./release-all.sh --major

release-dry-run:
	./release-all.sh --patch --dry-run
