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
	@echo "Generating README.md from README-raw.md..."
	@./target/release/thingy help > .help.tmp 2>&1
	@awk '/\{\{USAGE\}\}/ { print "```"; while(getline line < ".help.tmp") print line; print "```"; next } { print }' README-raw.md > README.md
	@rm -f .help.tmp
	@echo "README.md generated"
