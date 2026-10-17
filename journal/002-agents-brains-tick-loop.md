# 002 — Agents, brains and the tick loop

**Goal:** things that move, eat, starve and die — and a clock to drive them.

## What we built

- `agent.rs`: `Direction` (+ `ALL`, `delta()`), `Action`, `Agent`, `MAX_ENERGY`.
- `brain.rs`: the `Brain` trait, and `SmoothBrain` — no wrinkles, no thoughts.
  Eats if standing on a berry, otherwise stumbles in a random direction.
- `sim.rs`: `Sim { world, agents, brains }` and the three-phase `tick()`.
- `render.rs`: `draw_agents` — a circle plus an energy bar per agent.
- `main.rs`: a fixed-timestep accumulator, run/pause, step, and a ticks/s slider.

Energy: start 100, max 150, −1 per tick, +25 per berry. Hit 0 and `alive = false`.

## Decisions

### `Action` is inert data, not a method call

The tempting design is `fn move_north(&mut self, world: &mut World)` — the brain calls
it, the effect happens. Rejected. An `Action` is a **value describing an intent**;
something else carries it out later. That buys four things:

- **The LLM's output maps straight onto it.** Parsing `{"action":"move",...}` produces
  an `Action` and nothing else happens yet.
- **It's loggable.** `Debug` on `Action` makes the event feed one `format!` away.
- **It's validatable.** An agent can *want* to walk into a wall; the simulation decides
  whether it happens. If brains mutated the world directly, a hallucinating model could
  teleport.
- **It's replayable.** A list of actions per tick is a complete recording of a run.

`Brain::decide` takes `&World` (shared, not `&mut`), so a brain *physically cannot*
change anything. The compiler enforces "brains only advise".

### `Box<dyn Brain>` — dynamic dispatch on purpose

`Vec<RandomBrain>` holds exactly one type. `Vec<Box<dyn Brain>>` means each agent's
brain is decided at runtime, so chunk 4 is a one-line change and — more usefully — a
**mixed population** becomes possible: two wrinkled brains and three smooth ones in the
same world, which is the actual experiment. Cost is a vtable lookup per decision,
irrelevant next to a network round-trip.

`decide` takes `&mut self` even though `SmoothBrain` has no state. `LlmBrain` will
carry conversation history and a client; declaring `&self` now means changing the trait
and every impl later.

### The three-phase tick — the important one

The obvious version doesn't compile:

```rust
for agent in &mut self.agents {
    let action = self.brains[agent.id].decide(&self.world, agent);
    self.apply(agent.id, action);   // error[E0499]: cannot borrow `*self` as
}                                    // mutable more than once at a time
```

The loop holds `&mut self.agents` for its whole duration; `self.apply()` wants all of
`self`. **Do not fight this with indices and scoping** — the borrow checker is pointing
at a real modelling bug. If agents acted one at a time inside the loop, Ada (id 0) would
move *before* Bo (id 1) decided, so low ids would react to a stale world and get
systematically better outcomes. That's an order bias baked into every emergent behaviour
you'd later observe, and you'd never notice it.

The split fixes both at once:

1. **decide** — holds only `&self.agents`, so everyone decides against the *same*
   snapshot. Simultaneous turns, no bias.
2. **apply** — the agents borrow is gone (`decisions` is an owned `Vec` of `Copy`
   values), so `&mut self` is free.
3. **metabolism** — drain energy, mark the dead, then `world.tick += 1`.

`decisions` isn't a workaround; it's the correct model of "everyone acts at once", and
the compiler made us find it. **Collect what you intend to do, then do it** is the
standard answer to most "cannot borrow as mutable more than once" errors.

This is commented in the code so nobody later "simplifies" it back into one loop.

### Why the borrow checker allows phase 1 at all

It tracks **individual fields** of `self` within a method body, so `&self.agents` and
`&mut self.brains[..]` coexist fine. The analysis only collapses when you go through a
*method* like `self.apply()`, which opaquely claims all of `self`.

### `id` as index

`agents[i].id == i`, and `brains[i]` belongs to `agents[i]`. Dead agents get
`alive = false` rather than being removed, because `Vec::remove` shifts every later
index and would silently pair agents with the wrong brains. A `HashMap` would be more
robust; a flat `Vec` with a tombstone is simpler and the invariant is easy to hold as
long as nothing is ever removed.

### Fixed timestep, not one tick per frame

Ticking once per frame ties simulation speed to the monitor — 60Hz means 60 ticks/s,
and the world runs 2.4× faster on someone's 144Hz screen. Instead, bank elapsed real
time and spend it in fixed chunks:

```rust
accumulator += get_frame_time();
while accumulator >= interval {   // while, not if
    sim.tick();
    accumulator -= interval;
}
```

A `while` means a 300ms frame hitch catches up by running two ticks, so sim time tracks
wall-clock time. `accumulator = 0.0` on pause stops a ten-second pause from banking ten
seconds of debt and firing 40 ticks the instant you resume.

**Hazard:** if one tick ever takes longer than `interval`, the accumulator grows faster
than the loop drains it — the *spiral of death*. Needs a catch-up cap before chunk 4,
where one tick means several network calls.

### `Action::Wait` is unused, and that's fine

`SmoothBrain` never emits it, so there's a dead-code warning. Leave it: `Wait` is the
**safe fallback for a brain that can't decide** — malformed JSON, an unknown action
name, a timeout, a 429. `parse(..).unwrap_or(Action::Wait)`. Falling back to a *random*
direction instead would make a broken integration look like a working one, which is the
worst possible failure mode.

Not worth emitting from `SmoothBrain` just to silence the warning: waiting costs energy
and gains nothing, so it would only weaken the baseline we want to measure against.

## Rust learned

- `impl Trait for Type` — trait first, type second. Read it as English: "implement
  Brain for SmoothBrain".
- A bare trait name is **not a type**. `dyn Brain` is how you say "some value
  implementing Brain" — which is why it's `Box<dyn Brain>`. Hard error since edition
  2021, and a good one: `dyn` marks where you pay for dynamic dispatch.
- Trait impls must match the declared signature exactly; extra methods go in a separate
  plain `impl Type` block. A type can have as many `impl` blocks as it likes.
- Prefer `&[T]` over `&Vec<T>` in signatures (and `&str` over `&String`). `&Vec` demands
  a heap-allocated growable vector specifically; `&Vec<T>` auto-derefs to `&[T]` at the
  call site anyway.
- Copying a small `Copy` field out of a struct **ends the borrow immediately**:
  `let target = self.agents[id].pos.step(dir);` frees you to call `self.agents.iter()`
  on the next line. This is the standard escape hatch, and it's why `Pos: Copy` matters.
- Enum variants carrying data (`Move(Direction)`) — a tag plus a payload, 2 bytes total.
  In C this is a struct with a union and a discriminant you must keep in sync by hand.
- Associated consts (`Direction::ALL`) stand in for the variant enumeration Rust doesn't
  have built in (that needs a derive macro like `strum`).

## Open questions for chunk 3

- `Brain::decide` currently receives the **entire `World`**. Fine for a random walker,
  wrong for an LLM: an agent shouldn't have omniscient knowledge of a map it hasn't
  explored, and the whole grid won't fit in a prompt economically anyway. Chunk 3
  replaces that parameter with an `Observation` — a limited, local view. Left deliberately
  wrong here so the reason for the change is felt rather than asserted.
- Grid size is bounded by context budget, not by screen: 32×22 ≈ 700 chars ≈ 200 tokens
  per observation. A 100×100 world would be ~10k tokens per agent per tick.

## Baseline to beat

`SmoothBrain`, seed 1, 4 agents: deaths at ticks **100, 300, 375 and 395** (wipeout),
for a total of **1170 agent-ticks**.

Reading those numbers:

- Energy starts at 100 and drains 1/tick, so **100 is the no-food floor**. First death
  landing exactly there means that agent ate *nothing* — it failed to stumble onto any
  of 48 berries in 100 random moves. That's the cost of having no strategy at all.
- The 770 agent-ticks above that floor is at least ~31 berries at +25 each — a lower
  bound, since berries eaten near the cap lose part of their value.
- Ceiling if all 48 berries were eaten at full value: 400 + 1200 = **1600 agent-ticks**,
  so a thinking brain has ~430 agent-ticks of headroom.

### Two findings that matter for chunk 4

**Berries never regrow.** The world is a depleting resource, so wipeout is guaranteed
no matter how good a brain is — a smart agent cannot "win", only last longer. That makes
this a clean benchmark (fixed resource, fixed seed, one number), but it also means a
long-running watchable terrarium will eventually need regrowth. Keep the fixed supply
for measurement; add slow regrowth later for the fun version, and record both.

**`MAX_ENERGY` makes *when* to eat a real decision.** `SmoothBrain` eats whenever it's
standing on a berry — including at energy 149, where the cap throws away 24 of the 25.
A brain that holds off until energy drops below 125 captures the full value from every
berry it finds. Same berries, more ticks, purely from timing.

That's a concrete, falsifiable hypothesis for `LlmBrain` to be tested against,
rather than a vague "the LLM should do better". It also happens to be the kind of
restraint an LLM plausibly works out from the rules, which makes it a genuinely
interesting first thing to watch for.
