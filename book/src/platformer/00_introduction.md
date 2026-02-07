# Learn agb part II: Platformer

In this tutorial, you'll learn how to make a multi-level platformer using `agb` and [Tiled](https://www.mapeditor.org/).

By the end, you'll have a game where a wizard runs and jumps across tile-based levels, colliding with the terrain, and advancing through multiple stages.

We'll cover:

- Designing levels in Tiled with custom tile properties.
- Using `build.rs` to parse and convert Tiled levels at compile time.
- Displaying a scrollable tile-based level on screen.
- Implementing player movement with gravity and friction.
- Tile-based collision detection and response.
- Supporting multiple levels with win detection.

**This tutorial assumes you've completed the [pong tutorial](../pong/00_introduction.md).** We'll build on concepts introduced there — sprites, backgrounds, fixed-point numbers, and input handling — and link back to those chapters when relevant.

Each chapter ends with an optional exercise.
Later chapters assume you haven't done them, so if you do try an exercise, revert those changes before starting the next chapter.
