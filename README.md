# randname

A command-line tool that generates pronounceable, fantasy-style names
by stitching together syllable fragments (a leading consonant cluster,
a vowel sound, and an optional trailing cluster).

I wanted a quick source of throwaway character and place names for
tabletop prep and for test fixtures, without pulling in a static name
list or hitting an API, and without the flakiness of unseeded
randomness that makes fixture output hard to reproduce across test
runs.

## Usage

Build and run with cargo:

```
cargo build --release
./target/release/randname
```

With no flags it prints 5 two-syllable names using a seed derived
from the current time:

```
$ randname
Thendor
Kabrai
Selthi
Draxun
Fyrion
```

Flags:

```
--count N, -c N       how many names to print (default: 5)
--syllables N, -s N   syllables per name (default: 2)
--seed N               fix the seed for reproducible output
```

Example, five three-syllable names with a fixed seed so the output is
the same on every run:

```
$ randname --count 5 --syllables 3 --seed 12345
```

## Design

The name-building logic in `src/namegen.rs` and the flag parsing in
`src/cli.rs` are both made of pure functions: given the same
arguments they always return the same result, with no I/O and no
mutable shared state. The only place that touches the system clock or
process id is `default_seed` in `src/main.rs`, and that function
exists purely to produce a starting seed - everything downstream of
that seed is deterministic and covered by unit tests.

That split is why `--seed` works: pass the same seed twice and you
get the same batch of names back.

## Testing

```
cargo test
```

No third-party crates are used, so `cargo test` only needs the
standard library.
