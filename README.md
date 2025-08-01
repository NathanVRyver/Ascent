# Project Ascent

A physics-based flight simulator for human-scale personal flight.

## What is this?

Ever wondered if humans could fly with the right wings? This simulator lets you experiment with different wing configurations and see what it would take to get a person airborne.

Built in Rust with real physics calculations - no magic, just lift equations and Newton's laws.

## Quick Start

```bash
cargo run
```

## Controls

- **Space** - Start/pause simulation
- **W/S** - Adjust wing angle
- **T** - Apply thrust (hold it down)
- **R** - Reset to starting position
- **Arrow keys** - Move camera around

## What am I looking at?

The colored lines show forces:
- Green = Lift
- Red = Gravity 
- Yellow = Drag
- Blue = Thrust
- White = Total force

If the green line is longer than the red one, you're going up!

## Current Setup

- 80kg human (average adult)
- 6 meter wingspan
- Basic wing profile
- Sea level air density

## Want to experiment?

The code is modular - you can easily tweak:
- Wing size and shape in `systems.rs`
- Mass and physics parameters
- Add new forces or control systems

## Future Ideas

- Variable wing geometry
- Energy/battery simulation for powered flight
- Different takeoff methods (running start, vertical, etc)
- Landing gear and ground dynamics

## Why?

Because the dream of personal flight isn't going away, and understanding the physics is the first step to making it real.