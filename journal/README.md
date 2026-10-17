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

Factual errors — a misread number, a wrong description of the code — are a different
thing, and get fixed in place in the original entry.

Each entry ends with **open questions** for the next chunk. Entry 003's are the live
ones: LLM calls must stay off the render thread, and the clock needs a catch-up cap.

## Baseline to beat

`SmoothBrain`, seed 1, 4 agents:

| Agent | Died at tick |
|---|---|
| Ada | 100 |
| Bo | 300 |
| Dot | 375 |
| Cy | 395 (wipeout) |
| **Total** | **1170 agent-ticks** |

Total agent-ticks is the headline number: it counts every agent, not just the luckiest
and unluckiest. Across 4 agents the theoretical ceiling is 400 + (48 × 25) = **1600**
agent-ticks if every berry were eaten at full value, so a thinking brain has ~430 of
headroom.

**First death at exactly 100 means that agent never ate a single berry** — energy
starts at 100 and drains 1/tick, so 100 is the no-food floor. One of four random
walkers failed to stumble onto any of 48 berries in 100 moves.

The 770 agent-ticks above that floor is at least **31 berries** at +25 each — a lower
bound, since berries eaten near `MAX_ENERGY` lose part of their value to the cap.

The run is fully deterministic: every random draw comes from one generator seeded by
`srand(seed)`, so these numbers are exact, not typical. They were verified identical
at `fe6bd35` (chunk 2), `b39fc59` (chunk 3) and after the `try_tick` refactor (chunk 4).

`SmoothBrain` isn't scaffolding to delete once the LLM works; it's the control group.
