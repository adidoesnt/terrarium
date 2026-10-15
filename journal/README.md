# Journal

A build log. Each entry records **what was built, which decisions were made, and why**
— the *why* being the part that's invisible in the diff six months from now.

For what the project is and how to run it, see the [root README](../README.md).

## Entries

| # | Entry | Covers |
|---|---|---|
| 001 | [The world and the renderer](001-world-and-renderer.md) | grid, tiles, generation, macroquad window, egui panel |
| 002 | [Agents, brains and the tick loop](002-agents-brains-tick-loop.md) | `Agent`, `Action`, the `Brain` trait, the three-phase tick, fixed timestep |
| 003 | [What the agent actually knows](003-observation.md) | `Observation`, the 9×9 window, relative coordinates, the text a model reads |

## Conventions

Entries are numbered in build order and never rewritten — if a later chunk overturns
an earlier decision, the new entry says so and links back. The point is to keep the
reasoning as it was *at the time*, including the parts that turned out to be wrong.

Each entry ends with **open questions** for the next chunk. Entry 002's are the live
ones: `Brain::decide` still receives the entire `World`, which is wrong for an LLM and
is what chunk 3 fixes.

## Baseline to beat

`SmoothBrain`, seed 1, 4 agents:

| Measure | Ticks |
|---|---|
| First death | 100 |
| Total wipeout | 450 |

**First death at exactly 100 means that agent never ate a single berry** — energy
starts at 100 and drains 1/tick, so 100 is the no-food floor. One of four random
walkers failed to stumble onto any of 48 berries in 100 moves.

The last survivor reached 450, i.e. ~350 ticks of surplus, i.e. roughly **14 berries**
at +25 each. Across 4 agents the theoretical ceiling is 400 + (48 × 25) = 1600
agent-ticks if every berry were eaten at full value.

See [002](002-agents-brains-tick-loop.md#baseline-to-beat) for what this implies about
the headroom available to a thinking brain.

`SmoothBrain` isn't scaffolding to delete once the LLM works; it's the control group.
