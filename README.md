# rustre

A Lustre transpiler to Rust.

## Usage

To transpile a simple example:

```shell
cargo run <test/simple.mls >test/simple.rs
```

To transpile, compile and run a simple example:

```shell
cd test
make simple
./simple
```

## Architecture

When transpiling a Lustre file, Rustre applies these steps:

1. Parsing (see `parser.rs` and `lustre.pest`): build an raw AST (see `ast.rs`)
   from an input Lustre file
2. Normalization (see `normalizer.rs`): build a normalized AST (see `nast.rs`)
   from a raw AST
3. Static scheduling (see `sequentializer.rs`): re-order equations in nodes so
   that they can be executed sequentially
4. Code generation (see `rustfmt.rs`): write Rust code from the AST, generate
   the necessary structures and logic for the `fby` operator

Each file contains a head comment which explains in detail what it does.

## License

MIT
