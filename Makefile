BINDIR ?= ~/bin

.PHONY: all
all: target/release/thingy

target/release/thingy: Cargo.toml src/main.rs
	cargo build --release

.PHONY: install
install: target/release/thingy
	mkdir -p $(BINDIR)
	cp target/release/thingy $(BINDIR)/thingy

.PHONY: clean
clean:
	cargo clean

.PHONY: readme
readme: target/release/thingy
	@echo "Updating README.md with current help output..."
	@./target/release/thingy help > .help.tmp 2>&1
	@grep -n "<!-- BEGIN AUTO-GENERATED -->" README.md | cut -d: -f1 > .line1.tmp
	@grep -n "<!-- END AUTO-GENERATED -->" README.md | cut -d: -f1 > .line2.tmp
	@head -n $$(cat .line1.tmp) README.md > README.md.tmp
	@cat .help.tmp >> README.md.tmp
	@echo "" >> README.md.tmp
	@tail -n +$$(cat .line2.tmp) README.md >> README.md.tmp
	@mv README.md.tmp README.md
	@rm -f .help.tmp .line1.tmp .line2.tmp
	@echo "README.md updated"
