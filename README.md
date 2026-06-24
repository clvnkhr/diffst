# diffst

`diffst` is a Typst package for presentable side-by-side document diff reports.

```typst
#import "lib.typ": diffst

#diffst("old.typ", "new.typ")
```

The Typst package reads both files, passes their contents to a Rust WebAssembly
plugin, and renders the structured diff as a side-by-side report.

## Build

```sh
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown
typst compile --root . examples/basic.typ
typst compile --root . examples/realistic.typ
```

## Options

```typst
#diffst(
  "old.typ",
  "new.typ",
  ignore-whitespace: true,
  show-whitespace: true,
  display: "collapsed", // or "full"
)
```

`show-whitespace` makes changed spaces and tabs visible inside inline highlights.
