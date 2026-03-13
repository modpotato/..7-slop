# ..7-slop

An implementation blueprint for a hyper-performant native UI framework in pure Rust + `wgpu`.

## Goal

Build a framework that compiles to a single native binary, minimizes memory use, and targets smooth 144hz+ rendering with strong battery efficiency.

## Core Stack (Use Existing Best-in-Class Crates)

Do not write these from scratch:

- **Windowing + events:** [`winit`](https://github.com/rust-windowing/winit)
- **Layout:** [`taffy`](https://github.com/DioxusLabs/taffy)
- **Text shaping + rendering:** [`cosmic-text`](https://github.com/pop-os/cosmic-text) + [`glyphon`](https://github.com/grovesNL/glyphon)

This keeps implementation effort focused on UI architecture and rendering performance instead of rebuilding foundational systems.

## Rendering Architecture (`wgpu`)

### 1) Batch Everything via Instancing

Use one quad pipeline and one instanced draw for most primitives.

```rust
#[repr(C)]
struct QuadInstance {
    position: [f32; 2],
    size: [f32; 2],
    color: [f32; 4],
    border_radius: f32,
    border_width: f32,
    uv_min: [f32; 2],
    uv_max: [f32; 2],
}
```

- Store all visible quads in a single instance buffer.
- Render with one instanced call (e.g. `draw(0..4, 0..instance_count)`).
- Minimize CPU-GPU sync and pipeline/state changes.

### 2) SDF-Based Styling in Fragment Shader

Use Signed Distance Field logic in WGSL for:

- rounded corners
- anti-aliased edges
- borders

This avoids CPU tessellation for curved geometry and maintains smooth visual quality.

### 3) Texture Atlas Strategy

- Pack icons/images into shared atlases.
- Include UVs in `QuadInstance`.
- Render textured and untextured quads in the same batch where possible.

### 4) Scissor Rects for Scroll/Clipping

For scroll containers and clipped regions, set scissor rectangles with:

- `wgpu::RenderPass::set_scissor_rect`

This prevents overdraw outside container bounds.

## UI Paradigm: Retained + Reactive

Prefer retained/reactive architecture over immediate mode for lower idle power usage.

- Maintain a persistent widget tree.
- Mark nodes dirty when state changes.
- Recompute layout only for affected regions.
- Rebuild/re-upload render instances only when needed.

Recommended tree storage:

- [`slotmap`](https://github.com/orlp/slotmap) for generational node IDs and flexible parent/child storage.
- [`indextree`](https://github.com/saschagrunert/indextree) when you want a tree-first API with explicit hierarchical traversal.

## Implementation Plan

### Phase 1 — Window + GPU Init

- Create a `winit` window.
- Initialize `wgpu` surface/device/queue.
- Render a clear color frame.

### Phase 2 — Quad Engine

- Add quad pipeline + instance buffer.
- Add WGSL SDF logic for rounded corners/borders.
- Performance test at high primitive counts (e.g. 100k overlapping quads) and realistic widget hierarchies.
- Validate concrete targets (for example: 144hz+ in typical UI scenes, no severe frame-time spikes during resize/scroll, and graceful degradation under synthetic stress tests).

### Phase 3 — Layout + Text

- Integrate `taffy` layout tree and map results to quad instances.
- Integrate `glyphon`/`cosmic-text` and render text over quads.

### Phase 4 — Input + Event Routing

- Route `winit` input through hit-testing against layout bounds.
- Support hover/click and local state updates.

### Phase 5 — Developer-Facing API

Expose ergonomic widget composition API (builder/macro/Elm-like).

```rust
VStack::new()
    .push(Text::new("Hello World"))
    .push(Button::new("Click me").on_click(|| println!("Clicked!")))
    .padding(10.0)
```

## Inspiration and References

- [Xilem / Vello](https://github.com/linebender/xilem)
- [iced](https://github.com/iced-rs/iced)
- [Lapce / Floem](https://github.com/lapce/lapce)

---

By combining `winit`, `taffy`, `cosmic-text`, and a custom instanced-SDF `wgpu` renderer, this project can provide a native Rust UI stack with high throughput and low overhead.

## Current Lightweight Core

This repository now includes a minimal trait-first core crate (`slop_ui`) that is intentionally lightweight and fully feature-flagged.

- `input`: enables input event dispatch and hit-testing helpers.
- `layout`: enables pluggable layout via a `LayoutEngine` trait.
- `text`: enables optional text collection/render hooks.

Example:

```bash
cargo test
cargo test --all-features
```
