# NeoView
NeoView is a lightweight, modern declarative UI framework that prioritizes robustness, safety, and efficiency over complex runtime magic.

Aligned with Rust's core principles, NeoView offers a practical middle ground in declarative UI design. It supports ergonomic, fully reactive UI definitions with a strong emphasis on safety, robustness, efficiency, and renderer agnosticism.

# Renderers
NeoView is renderer-agnostic, it supports any platform or renderer, provided they implement the necessary items.

The available renderers include:
- [`neoview-web`](./web/): A renderer targeting the web platform based on HTML and the DOM.