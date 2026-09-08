// Deterministic name generation. Every function here is pure: same
// inputs always produce the same output, with no I/O and no shared
// mutable state. That's what lets the generation logic be unit
// tested without mocking a random number source or capturing stdout.

const ONSETS: &[&str] = &[
    "b", "br", "c", "ch", "d", "dr", "f", "fr", "g", "gr", "h", "j", "k", "kr", "l", "m", "n",
    "p", "pr", "r", "s", "sh", "st", "t", "th", "tr", "v", "w", "y", "z",
];

const NUCLEI: &[&str] = &["a", "ae", "ai", "e", "ea", "i", "io", "o", "oa", "oi", "u", "y"];

const CODAS: &[&str] = &["d", "l", "ld", "lin", "n", "nd", "r", "rin", "s", "sh", "th", "x"];

/// One step of the splitmix64 generator: given a state, returns the
/// next state. Deterministic and pure, which is what makes it usable
/// both for seeding and for writing tests with known fixed points.
fn splitmix64(state: u64) -> u64 {
    let mut z = state.wrapping_add(0x9E3779B97F4A7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^ (z >> 31)
}

/// Builds one capitalized name from a seed and a syllable count. The
/// same seed and syllable count always produce the same name, which
/// is what makes the CLI's --seed flag give reproducible output.
pub fn generate(seed: u64, syllable_count: usize) -> String {
    let mut state = seed;
    let mut name = String::new();
    let syllables = syllable_count.max(1);

    for _ in 0..syllables {
        state = splitmix64(state);
        let onset = ONSETS[(state % ONSETS.len() as u64) as usize];

        state = splitmix64(state);
        let nucleus = NUCLEI[(state % NUCLEI.len() as u64) as usize];

        name.push_str(onset);
        name.push_str(nucleus);

        state = splitmix64(state);
        if state % 3 != 0 {
            state = splitmix64(state);
            let coda = CODAS[(state % CODAS.len() as u64) as usize];
            name.push_str(coda);
        }
    }

    capitalize(&name)
}

/// Generates `count` names starting from `seed`, advancing the seed
/// deterministically between names so the whole batch is reproducible
/// from that one starting seed.
pub fn generate_batch(seed: u64, count: usize, syllable_count: usize) -> Vec<String> {
    let mut names = Vec::with_capacity(count);
    let mut state = seed;
    for _ in 0..count {
        names.push(generate(state, syllable_count));
        state = splitmix64(state);
    }
    names
}

/// Uppercases the first character of `s`, leaving the rest untouched.
pub fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_gives_same_name() {
        assert_eq!(generate(42, 2), generate(42, 2));
    }

    #[test]
    fn different_seeds_usually_differ() {
        assert_ne!(generate(1, 2), generate(2, 2));
    }

    #[test]
    fn zero_syllables_is_treated_as_one() {
        assert_eq!(generate(7, 0), generate(7, 1));
    }

    #[test]
    fn batch_has_requested_length() {
        assert_eq!(generate_batch(1, 5, 2).len(), 5);
    }

    #[test]
    fn batch_is_reproducible_from_same_seed() {
        assert_eq!(generate_batch(9, 4, 2), generate_batch(9, 4, 2));
    }

    #[test]
    fn capitalize_handles_empty_string() {
        assert_eq!(capitalize(""), "");
    }

    #[test]
    fn capitalize_uppercases_first_letter_only() {
        assert_eq!(capitalize("thalindor"), "Thalindor");
    }

    #[test]
    fn names_start_uppercase() {
        let name = generate(123, 3);
        assert!(name.chars().next().unwrap().is_uppercase());
    }
}
