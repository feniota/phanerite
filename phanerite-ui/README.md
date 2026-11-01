# PhaUI

PhaUI (**Pha**nerite **UI**) is a rather simple Rust UI library.

`phanerite-executor` owns native windows, platform events and the window GPU
lifecycle: surface acquisition, command submission and presentation.
`phanerite-layout` provides `LayoutTree`, a retained box-layout tree for a logical
viewport, and `LayoutNodeKind::{Box, Text, Replaced}`. The layout tree is separate
from the executor's native `window::Window`.
`phanerite-render` defines `RenderContent` and `RenderViewport`, and draws owned
layout snapshots using wgpu. It does not depend on the executor, Tao or GTK;
the same drawing code works with a window target or an offscreen texture.

The render crate's default `layout-debug` feature supplies `LayoutDebugRenderer`
and its layout dependency. The executor disables that feature and only uses the
shared drawing interface and viewport; its library does not depend on Taffy or
layout trees. The layout example enables the feature and composes all three
crates. GPU readback tests live in `phanerite-render`.

`Text` reserves a text flow rather than an individual source `span`.
`Replaced` identifies externally rendered content such as an image, SVG or 3D
scene; an ordinary inline-block container is still a `Box`. Rich-text runs and
inline-box integration are planned and are not implemented by these node kinds.

Run the layout inspector from the repository root:

```sh
toolbox run -c rust cargo run -p phanerite-executor --example layout
```

The example draws nested flex and grid boxes with percentage and fractional
sizes. Resize the window to recompute layout. Outline colors indicate tree
depth. On Fedora, native compilation needs `gtk3-devel` and `dbus-devel`; a
Vulkan driver such as `mesa-vulkan-drivers` is needed for Vulkan rendering.
On Wayland, the GPU owns a dedicated subsurface covering the GTK content area;
GTK retains the parent surface and native decorations. This prevents GTK's
software buffers from conflicting with Vulkan's explicit synchronization. The
renderer owns the subsurface lifetime, position and physical content extent.

Layout coordinates, sizes, padding and gaps use fractional **logical pixels**.
The logical viewport is the physical client size divided by the native window
scale factor; `RenderViewport` carries that pair for a frame. Taffy rounding is
disabled. `LayoutTree::compute_layout` accumulates ancestor offsets in logical
space and returns an owned `LayoutSnapshot` with absolute border boxes. Detached
nodes and `display: none` subtrees are omitted. `needs_layout` reports Taffy's
root invalidation; changing tree input does not itself request a window redraw.

The rendering boundary is **CPU layout tree → owned snapshot → GPU instances**.
Snapshots contain node IDs, kinds, depths and logical rectangles; they hold no
component values, tree borrows, GPU resources or scale factors, and are
`Send + Sync`. The current executor and drawing content stay on one local
thread; moving layout or rendering to another thread should transfer snapshots.
`RenderContent::prepare` runs before frame encoding, so the example recomputes
and uploads only when layout or the physical-size/scale pair changes. A snapshot
must match the render viewport's logical dimensions; stale pairs are rejected.
Native resize and DPI notifications request redraws, and zero-size frames are
skipped. These are window lifecycle notifications, not a UI event system.

`LayoutDebugRenderer` uploads one 32-byte instance per visible, nonempty box:
`[logical_x, logical_y, width, height, linear_r, linear_g, linear_b, alpha]`.
A 16-byte uniform supplies physical viewport dimensions and scale. The shader
scales and rounds **absolute edges** once, then draws four inset strips as
triangles. Every outline is **one physical pixel** wide at every scale and does
not change layout. Instance buffers grow when necessary and are reused.
Here, a physical pixel means a render-target pixel. The scale comes from Tao;
GTK 3 can use integer buffer scaling on a fractionally scaled Wayland desktop,
where the compositor may resample the final window image.

This is a layout inspector: text and replaced content have zero intrinsic
measurement and no content painting; explicit sizes and normal box styles
still work. There is no rich-text shaping, ancestor overflow clipping, CSS
stacking, hit testing, component lifecycle or UI event propagation. Drawing is
parent-before-child and clipped only to the viewport.

Scoped verification (the render test and smoke test require a GPU adapter):

```sh
toolbox run -c rust cargo test -p phanerite-layout -p phanerite-render -p phanerite-executor
toolbox run -c rust cargo test -p phanerite-render --test layout_render -- --ignored --nocapture
toolbox run -c rust cargo run -p phanerite-executor --example layout -- --smoke-test
```

The smoke test presents three logical window sizes and closes automatically.
