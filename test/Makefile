RUSTC ?= rustc
CARGO ?= cargo
TARGETS = pendulum simple sujet tutorial

all: $(TARGETS)

%.rs: %.mls
	$(CARGO) run --manifest-path ../Cargo.toml <$< >$@

%: %.rs
	$(RUSTC) -o $@ $^

.PHONY: clean
clean:
	$(RM) -f $(TARGETS) *.rs
