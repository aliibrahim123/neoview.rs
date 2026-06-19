# NeoView Web
The web renderer for `neoview`.

`neoview-web` is the official web renderer for the [`neoview`](https://docs.rs/neoview) framework, based on [`HTML`](https://developer.mozilla.org/en-US/docs/Web/HTML) and [`DOM`](https://developer.mozilla.org/en-US/docs/Web/API/Document_Object_Model) technologies and interfacing through the `wasm-bindgen` crate.

Aligned with `neoview`'s core principles, `neoview-web` supports ergonomic, fully reactive UI definitions with a strong emphasis on safety, robustness, and efficiency.

Like every `neoview` renderer, it utilizes the efficiency of fine-grained reactivity, the safety of context passing, and the ergonomics of chunked templating to provide high-level expressiveness with low-level robustness.

# Features
`neoview-web` features its own context, `DomContext`, it borrows the reactive system from `neoview` and provides its own flavor of the `chunk` macro with full HTML support.

In addition to the feature-richness of `chunk`, `neoview-web` includes conditional rendering, list rendering, a builder pattern for templating, and IntelliSense for tags, attributes, events, and CSS properties.

Here is a simple example without the initialization boilerplate:
```rust
chunk!(build, div {
    h3 { "Hello world!" }
    do {
        let count = build.prop(0);
        chunk!(build, button(
            on.click: (move |ctx, _| ctx.update(count, |v| *v += 1))
        ) { "count: ", count });
    }
});
```

For a more in-depth introduction, check out the [guide section](https://docs.rs/neoview-web/latest/neoview_web/docs/guide).

# Crate Features
### `html-types`
Provides IntelliSense for HTML tags, attributes, and events (autocompletion and hover descriptions).

It is an optional, quality-of-life feature with no runtime cost.
### `css-types`
Provides IntelliSense for CSS properties (autocompletion and hover descriptions).

It is an optional, quality-of-life feature with no runtime cost.

It requires the `html-types` feature.