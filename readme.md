# Akari

A solver for [Akari](https://www.janko.at/Raetsel/Akari) Puzzles that uses the [z3](https://github.com/Z3Prover/z3) SMT prover.

```
curl -v --silent https://www.janko.at/Raetsel/Akari/523.a.htm 2>&1 | sed -n -e '/problem/,/solution/ p' | sed -e '1d;$d' | ./target/release/akari
```

## Table Of Contents

-   [Introduction](#introduction)
-   [Usage](#usage)
-   [Documentation](#docs)

<a name="introduction"></a>

## Introduction

> Akari (美術館, びじゅつかん, Bijutsukan, Light Up, Lighting) is a Japanese logic puzzle: place lamps in such a way that all squares are illuminated and lamps do not illuminate each other..

![Akari Example](example.png)

There are 3 rules for the puzzle:

1. Place light bulbs in some of the white cells so that all white cells are lit and no light bulb is lit by an other light bulb.
2. A light bulb shines horizontally and vertically up to the next black cell or the edge of the grid.
3. A number in a black cell indicates how many light bulbs must be placed in orthogonally adjacent cells.

<a name="usage"></a>

## Usage

Create a build using the following command:

```sh
cargo build --release
```

Pass the puzzle to be solved via stdin:

```sh
cat puzzles/074.txt | ./target/release/akari
```

A script for solving a puzzle from [www.janko.at](www.janko.at) using `curl`:

```sh
curl -v --silent https://www.janko.at/Raetsel/Akari/523.a.htm 2>&1 | sed -n -e '/problem/,/solution/ p' | sed -e '1d;$d' | ./target/release/akari
```

<a name="docs"></a>

## Documentation