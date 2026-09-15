// Deterministic name generation. Every function here is pure: same
// inputs always produce the same output, with no I/O and no shared
// mutable state. That's what lets the generation logic be unit
// tested without mocking a random number source or capturing stdout.

const COMMON_ONSETS: &[&str] = &[
    "b", "br", "c", "ch", "d", "dr", "f", "fr", "g", "gr", "h", "j", "k", "kr", "l", "m", "n",
    "p", "pr", "r", "s", "sh", "st", "t", "th", "tr", "v", "w", "y", "z",
];

const COMMON_NUCLEI: &[&str] = &["a", "ae", "ai", "e", "ea", "i", "io", "o", "oa", "oi", "u", "y"];

const COMMON_CODAS: &[&str] = &["d", "l", "ld", "lin", "n", "nd", "r", "rin", "s", "sh", "th", "x"];

const ELVISH_ONSETS: &[&str] = &[
    "l", "m", "n", "s", "v", "th", "cal", "fal", "gal", "sil", "fin", "thal", "quel", "mir",
];

const ELVISH_NUCLEI: &[&str] = &["a", "ae", "e", "ea", "i", "ia", "ie", "o", "y", "ye"];

const ELVISH_CODAS: &[&str] =
    &["l", "lin", "n", "ra", "riel", "th", "wen", "iel", "as", "ir"];

const DWARVISH_ONSETS: &[&str] = &[
    "b", "d", "g", "gr", "k", "kh", "thr", "dr", "br", "gor", "thor", "grum", "bal", "dur",
];

const DWARVISH_NUCLEI: &[&str] = &["a", "o", "u", "au", "ou", "i"];

const DWARVISH_CODAS: &[&str] =
    &["in", "ur", "gar", "rim", "dur", "g", "k", "m", "grim", "bak"];

/// The syllable palette used when generating a name. Each style pulls
/// from its own onset/nucleus/coda tables, so the same seed produces
/// a different-sounding name depending on the style requested.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Style {
    Common,
    Elvish,
    Dwarvish,
}

impl Style {
    fn tables(self) -> (&'static [&'static str], &'static [&'static str], &'static [&'static str]) {
        match self {
            Style::Common => (COMMON_ONSETS, COMMON_NUCLEI, COMMON_CODAS),
            Style::Elvish => (ELVISH_ONSETS, ELVISH_NUCLEI, ELVISH_CODAS),
            Style::Dwarvish => (DWARVISH_ONSETS, DWARVISH_NUCLEI, DWARVISH_CODAS),
        }
    }
}

impl std::str::FromStr for Style {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "common" => Ok(Style::Common),
            "elvish" => Ok(Style::Elvish),
            "dwarvish" => Ok(Style::Dwarvish),
            other => Err(format!(
                "unknown style: '{other}' (expected common, elvish, or dwarvish)"
            )),
        }
    }
}

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
pub fn generate(seed: u64, syllable_count: usize, style: Style) -> String {
    let (onsets, nuclei, codas) = style.tables();
    let mut state = seed;
    let mut name = String::new();
    let syllables = syllable_count.max(1);

    for _ in 0..syllables {
        state = splitmix64(state);
        let onset = onsets[(state % onsets.len() as u64) as usize];

        state = splitmix64(state);
        let nucleus = nuclei[(state % nuclei.len() as u64) as usize];

        name.push_str(onset);
        name.push_str(nucleus);

        state = splitmix64(state);
        if state % 3 != 0 {
            state = splitmix64(state);
            let coda = codas[(state % codas.len() as u64) as usize];
            name.push_str(coda);
        }
    }

    capitalize(&name)
}

/// Generates `count` names starting from `seed`, advancing the seed
/// deterministically between names so the whole batch is reproducible
/// from that one starting seed.
pub fn generate_batch(seed: u64, count: usize, syllable_count: usize, style: Style) -> Vec<String> {
    let mut names = Vec::with_capacity(count);
    let mut state = seed;
    for _ in 0..count {
        names.push(generate(state, syllable_count, style));
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
        assert_eq!(generate(42, 2, Style::Common), generate(42, 2, Style::Common));
    }

    #[test]
    fn different_seeds_usually_differ() {
        assert_ne!(generate(1, 2, Style::Common), generate(2, 2, Style::Common));
    }

    #[test]
    fn zero_syllables_is_treated_as_one() {
        assert_eq!(generate(7, 0, Style::Common), generate(7, 1, Style::Common));
    }

    #[test]
    fn batch_has_requested_length() {
        assert_eq!(generate_batch(1, 5, 2, Style::Common).len(), 5);
    }

    #[test]
    fn batch_is_reproducible_from_same_seed() {
        assert_eq!(
            generate_batch(9, 4, 2, Style::Common),
            generate_batch(9, 4, 2, Style::Common)
        );
    }

    #[test]
    fn same_seed_usually_differs_across_styles() {
        assert_ne!(generate(42, 2, Style::Common), generate(42, 2, Style::Elvish));
        assert_ne!(generate(42, 2, Style::Common), generate(42, 2, Style::Dwarvish));
    }

    #[test]
    fn style_parses_from_str_case_insensitively() {
        assert_eq!("elvish".parse::<Style>(), Ok(Style::Elvish));
        assert_eq!("Dwarvish".parse::<Style>(), Ok(Style::Dwarvish));
        assert_eq!("COMMON".parse::<Style>(), Ok(Style::Common));
    }

    #[test]
    fn style_rejects_unknown_name() {
        assert!("orcish".parse::<Style>().is_err());
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
        let name = generate(123, 3, Style::Common);
        assert!(name.chars().next().unwrap().is_uppercase());
    }
}
