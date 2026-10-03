.PHONY: build local

build:
	cargo run

# Build, then serve dist/ at http://localhost:8000
local: build
	python -m http.server 8000 -d dist
