# 001 — The world and the renderer

**Goal:** a grid world on screen, with a panel showing live stats. No agents, no time.

## What we built

- `World`: a `width × height` grid of `Tile` (`Empty`, `Wall`, `Berry`), plus a `tick`
  counter that nothing increments yet.
- Procedural generation: a solid wall border, scattered interior walls, scattered berries.
- `Layout` + `draw_world`: fit the grid to the available area and paint it.
- A macroquad window with an egui side panel: tick, size, berry count, wall count,
  and a `regenerate` button.

## Decisions

**Flat `Vec<Tile>`, not `Vec<Vec<Tile>>`.** The nested version is one heap allocation
per row, scattered in memory, with two pointer hops per access. Flat is one contiguous
block, trivially cloneable and serialisable. Cost: index math by hand, `y * width + x`
— "skip y whole rows, then walk x across".

**`Pos` holds `i32`, not `usize`.** The most load-bearing decision in the file. With
`usize`, an agent at `x = 0` moving west computes `0 - 1`, which panics in debug and
wraps to ~18 quintillion in release. Signed coords make "one west of the left edge"
an ordinary `Pos { x: -1 }` that `in_bounds` rejects calmly. The conversion to `usize`
happens in exactly one place — `index` — *after* the bounds check proves it safe.
General rule: push unsigned types to the boundary of the system, not through it.

**`get` returns `Option<Tile>`.** Forces every caller to answer "what does off-the-map
mean?". Movement code wants exactly this: `None` naturally means "can't go there".

**Agents are deliberately not a `Tile` variant.** A tile is what the terrain *is*; an
agent is a mobile entity with energy and a brain, and two of them can contest one cell
on the same tick. Modelling them as tiles makes that collision unrepresentable.

**`match` on `Tile` with no `_` arm.** Adding `Tile::Water` later becomes a compile
*error* pointing at every place that forgot to handle it. A catch-all arm throws away
Rust's best refactoring tool.

**`high_dpi: false`.** On a Retina Mac, `high_dpi: true` makes `screen_width()` return
physical pixels while egui lays out in logical points — the panel and grid then disagree
about where the boundary is. Off means one unit means one thing everywhere. Slightly
softer rendering; revisit later.

## Generation is darts, not placements

```rust
for _ in 0..(width * height) / 12 {
    let p = Pos::new(gen_range(..), gen_range(..));
    if world.get(&p) == Some(Tile::Empty) {   // berry darts can miss
        world.set(&p, Tile::Berry);
    }
}
```

Two darts can hit the same cell, and berry darts landing on a wall or an existing berry
are discarded. So the divisor sets an **upper bound**, not a count: 58 attempts yields
~48 berries. The alternative — retry until N are placed — risks an infinite loop on a
crowded map, so darts-with-a-filter is the right trade.

Densities (`/25` walls, `/12` berries) are eyeballed. What matters is the *ratio*:
berries roughly 2× walls, so food is findable. If food is rarer than obstacles,
everything starves and you end up debugging your prompt when the real problem is the map.

Expressed as a fraction of area rather than an absolute count so density stays constant
if the grid is resized.

**Observed on seed 1, 32×22:** 48 berries, 132 walls.

## Watch out for

At higher wall density, walls can seal off a pocket of the map. An agent spawned inside
starves no matter how clever its brain is, and you'd spend an hour blaming the prompt.
If wall density goes up, add a flood-fill connectivity check to `World::new`.

## Rust learned

- `mod world;` *declares* the module (compiles the file); `use` is only a shortcut.
  Without `mod`, the file is dead text — the #1 surprise coming from Python/JS.
- `#[derive(Clone, Copy, PartialEq, Eq, Debug)]` on small pointer-free types. `Copy`
  makes them behave like `i32` and removes almost all borrow-checker friction.
- Method calls auto-deref; **operators do not**. Hence `**t == tile` inside
  `.filter()`, where `t: &&Tile` — one `&` from `.iter()`, one from `filter` taking
  `&Self::Item`. The idiomatic fix is to destructure in the pattern: `|&&t| t == tile`.
- `const` is inlined at each use site and needs an explicit type; `static` is one fixed
  address. For small `Copy` palette values, `const` is correct.
- `pub` on a module item is the line between interface and internals. `COLOR_BACKGROUND`
  is `pub` because `main.rs` clears the screen with it; the tile colours stay private
  because nothing outside `render.rs` should know what a wall looks like.
- Explicit imports beat `use macroquad::prelude::*`: provenance stays visible and
  unused-import warnings stay meaningful.

## macroquad notes

- `#[macroquad::main]` generates a loop that polls your future once per frame.
  `next_frame().await` is a **frame boundary**, not concurrency — it is not tokio.
  Blocking in this loop freezes the window, which is why chunk 4's LLM calls must
  move off this thread.
- egui is **immediate mode**: the whole panel is re-declared every frame and
  `ui.button(..).clicked()` is a `bool` for *this* frame. Nothing can go stale, because
  nothing is retained.
- Ordering: `draw_world` → `egui_macroquad::ui()` (runs panel code, tessellates) →
  `egui_macroquad::draw()` (paints it) → `next_frame().await`.
