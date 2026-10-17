# 003 — What the agent actually knows

**Goal:** stop handing brains the whole world. Give them a limited, local, *sendable*
view — and be able to read it, exactly as the model will, before spending anything on
an API call.

## What we built

- `observation.rs`: `Observation`, built fresh each tick, plus an `impl Display` that
  renders it as the text a model will read.
- `Brain::decide(&mut self, obs: &Observation)` — the `&World` parameter is gone.
- `Sim::observe(id)`.
- Panel: a row of agent buttons (`selectable_value`) and a collapsing block showing
  the selected agent's observation in monospace. The selected agent is drawn blue
  instead of amber.

This supersedes the open question at the end of [002](002-agents-brains-tick-loop.md):
`decide` no longer receives the entire `World`.

## Three reasons the type exists

Any one of these would force the design on its own.

**1. Omniscience makes the experiment meaningless.** With `&World`, an agent knows
where every berry is, including ones behind walls it has never visited. Any "intelligent
foraging" that follows is just shortest-path over perfect information. The interesting
question — can it explore sensibly when it *doesn't* know? — becomes unaskable.

**2. Token budget.** The full 32×22 grid is ~700 characters per observation; four agents
at 4 ticks/s is ~11k tokens/second of input. The 9×9 window plus the nearby lists is
~150 tokens — roughly 5× cheaper, and a model attends to it far better than to a wall
of mostly-irrelevant map.

**3. It is owned data, which chunk 4 needs.** Note what is *absent* from the struct: no
`&World`, no `&Agent`, no lifetime parameters. `name: String` not `&str`;
`view: Vec<char>` not a slice. In chunk 4 an `Observation` gets sent down a channel to a
worker thread while the render loop carries on at 60fps. **Owned data crosses thread
boundaries; borrowed data does not** — not without scoped threads and a lot of lifetime
pain. A few allocations per tick buys a trivially threadable design.

General shape of the lesson: a type whose job is to *leave* its home — across a thread,
a channel, a network — should own its contents.

## The three coordinate systems

```
1. WORLD      absolute, what Pos holds          x: 0..32, y: 0..22
2. RELATIVE   offsets from the agent (dx, dy)   -4..=4
3. VIEW       index into the char buffer        row: 0..9, col: 0..9
```

`Observation::new` walks system 2, reads from system 1, writes into system 3.

```
// relative -> view index
row = dy + VISION_RADIUS        // -4..=4 becomes 0..=8
col = dx + VISION_RADIUS
index = row * view_side + col
// the agent is always dead centre: 4*9 + 4 = 40, which is 81/2
```

Side is `2 * VISION_RADIUS + 1` — radius each way **plus one for the agent itself**.
Vision sizes are always odd; an even-sided square has no centre cell.

**The one real trap:** the loop never computes that index, it just pushes in order, with
`dy` outer and `dx` inner to match row-major layout. Swapping the loops silently
transposes the entire map and nothing errors.

Near a map edge the window hangs off the side and those cells come back `' '` from the
`None` branch. Honest: off-map is not wall, it is nothing.

## Representation choices

**`Vec<char>`, not `String`.** A `String` is UTF-8 bytes and cannot be indexed by
character position — `s[40]` doesn't even compile. The whole point is random access to
"row 4, col 4". 81 chars × 4 bytes = 324 bytes, irrelevant here.

**`char`, not `Tile`.** The view holds things that aren't terrain: `@` for self, `A`–`Z`
for others, `' '` for off-map. Tradeoff acknowledged: this bakes presentation into the
data, where a purist would use a `ViewCell` enum plus a rendering step. For a view whose
only consumer is a text prompt, the data and its presentation genuinely are the same
thing. Revisit only if something non-textual ever reads it.

**Precedence is a layering order:** self, then agents, then terrain. An agent standing
on a berry renders `B`, and that berry is *not* recorded in `berries`, because the push
only happens in the terrain branch. Deliberate — you can't see what someone is standing
on — and it quietly creates a reason to watch where others linger.

## The decision that matters most: relative coordinates

```rust
pub berries: Vec<(i32, i32)>,   // offsets from the agent, not map positions
```

Compare what you could tell a model:

> You are at (17, 9). There is a berry at (14, 11).

against what `describe_offset` produces:

> Berries you can see:
>   - 3 west, 2 south

The first makes the model do coordinate arithmetic before it can act, and that is
exactly what LLMs are unreliable at — off-by-ones, sign errors, confusion about whether
y grows up or down. The second is directly actionable: "3 west" maps onto `Move(West)`
with no intermediate step. The arithmetic moved from the unreliable component into the
reliable one.

Absolute `Pos` still exists on `Agent` for the simulation and for debugging. It is just
not what the agent is *told*. **Give a model the frame of reference it can act in, not
the one your data structure happens to use.**

Same reasoning behind spelling out `"You lose 1 every tick. At 0 you die."` — a model
cannot infer the rules of a world it has never played.

## Why the map *and* the lists

The observation states the same information twice, on purpose.

- The **map** carries spatial gestalt: walls forming a corridor, a cluster to the
  northeast, whether you are boxed in.
- The **lists** carry precision: exact, unambiguous, already in words that map onto
  actions.

Models read "3 west, 2 north" better than they count grid cells, and read enclosure from
a picture better than from a list. Both plays to a different strength, for ~40 tokens.

## Rust learned

- `impl fmt::Display` rather than a `to_prompt()` method: you get `.to_string()`,
  `format!("{obs}")` and `println!("{obs}")` free, and it is where any Rust reader looks
  first. `fmt::Result` is `Result<(), fmt::Error>`; the `?` after each `writeln!`
  propagates write errors and the trailing `Ok(())` is the success case.
- `Display` is for human-facing text — which this is, for a reader who happens to be a
  model. JSON for the API would be `Serialize`, a different trait; both can coexist.
- `self.view[start..end]` is a **slice**, a borrowed window, no copying.
  `.iter().collect::<String>()` works because `String: FromIterator<char>`.
- Changing a trait signature breaks every impl and call site at once, and `cargo check`
  enumerates them. A compiler-driven refactor is the thing Rust is genuinely best at —
  nothing like finding these at runtime, one agent at a time.
- `Vec::with_capacity` when the final size is known up front: one allocation instead of
  a growth sequence.

## Deliberately not built

- **Memory.** An `Observation` is a message, not a component: built in phase 1, read by
  one brain, dropped. Agents are completely amnesiac — a wrinkled brain that can't
  remember "I already searched west" will wander in circles. This is why
  `Brain::decide` takes `&mut self`: memory belongs in the *brain*, and that's chunk 5.
- **The vision overlay** (a rectangle on the world showing the 9×9 window) and
  click-to-select. Worth adding when "what could it actually see?" becomes the first
  question of every debugging session — which it will, in chunk 4.

## Open questions for chunk 4

- An agent only knows where the map edge is if it is within 4 tiles. Otherwise the
  window looks identically open in every direction. Is that right, or should distance
  to the edge be stated?
- Agents don't know berries never regrow. Should they?
- One tick now means N API calls. The fixed-timestep accumulator in `main.rs` will
  spiral if a tick outlasts its interval — it needs a catch-up cap, and the LLM work
  needs to be off the render thread entirely.
