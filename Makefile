# Convenience wrapper around cargo. Run `make` (or `make help`) for the list.

.DEFAULT_GOAL := help
.PHONY: help build run test web serve deploy clean

PORT ?= 8000

# The browser version is the same program built for WebAssembly: macroquad
# runs there as it is, loaded by its own small JavaScript loader (no
# wasm-bindgen). The loader is fetched once into web/ (commit it to pin it);
# delete it to fetch it again.
WASM_TARGET := wasm32-unknown-unknown
MQ_JS_BUNDLE_URL := https://not-fl3.github.io/miniquad-samples/mq_js_bundle.js

help:
	@echo "rock — common tasks"
	@echo
	@echo "  make build    cargo build --release"
	@echo "  make run      cargo run --release — the game in a window"
	@echo "  make test     cargo test"
	@echo "  make web      build the browser version (WebAssembly) into web/dist"
	@echo "  make serve    make web, then serve it at http://localhost:$(PORT)"
	@echo "  make deploy   make web, then deploy web/dist to Cloudflare (needs CLOUDFLARE_API_TOKEN)"
	@echo "  make clean    remove the builds"

build:
	cargo build --release
run:
	cargo run --release
test:
	cargo test

web/mq_js_bundle.js:
	curl -fsSL -o $@ $(MQ_JS_BUNDLE_URL)

# Its own target directory, whatever CARGO_TARGET_DIR or ~/.cargo/config.toml
# say, so the wasm is where this looks for it.
web: web/mq_js_bundle.js
	@rustup target list --installed | grep -qx $(WASM_TARGET) || rustup target add $(WASM_TARGET)
	cargo build --release --target $(WASM_TARGET) --target-dir target
	rm -rf web/dist
	mkdir -p web/dist
	cp target/$(WASM_TARGET)/release/rock.wasm web/index.html web/mq_js_bundle.js web/dist/
	@echo "Built web/dist. Serve it (make serve) and open it in a browser; it won't load from file://."

serve: web
	@echo "Rock at http://localhost:$(PORT)  (Ctrl-C to stop)"
	cd web/dist && python3 -m http.server $(PORT)

# rock.do.id.au is a Cloudflare Worker serving web/dist (wrangler.jsonc). The
# files keep their names from one build to the next, so browsers check for a
# newer copy each time (Cloudflare's default; unchanged ones come back 304).
deploy: web
	npx --yes wrangler@4 deploy

clean:
	cargo clean
	cargo clean --target-dir target
	rm -rf web/dist
