use lazy_static::lazy_static;
use regex::Regex;

static CONSONANTS: [char; 30] = [
    'b', 'ɓ', 'ʙ', 'β', 'c', 'd', 'ɗ', 'ɖ', 'ð', 'f', 'g', 'h', 'j', 'k', 'l',
    'ł', 'm', 'ɱ', 'n', 'ɳ', 'p', 'q', 'r', 's', 't', 'v', 'w', 'x', 'y', 'z'];
static VOWELS: [char; 36] = [
    'i', 'y', 'ɨ', 'ʉ', 'ɯ', 'u', 'ɪ', 'ʏ', 'ʊ', 'ɯ', 'ʊ', 'e', 'ø', 'ɘ', 'ɵ', 'ɤ', 'o', 'ø',
    'ə', 'ɵ', 'ɤ', 'o', 'ɛ', 'œ', 'ɜ', 'ɞ', 'ʌ', 'ɔ', 'æ', 'ɐ', 'ɞ', 'a', 'ɶ', 'ä', 'ɒ', 'ɑ'];

lazy_static! {
    static ref FULL_RE: Regex = Regex::new(r"([-+]{0,1})(\w+)\s{0,1}([\+\-][vc]){0,1}\s{0,1}([\+\-][vc]){0,1}").unwrap();
    static ref PREFIX_RE: Regex = Regex::new(r"(.+)(\-[vcVC]).*").unwrap();
    static ref SUFFIX_RE: Regex = Regex::new(r"(.+)(\+[vcCV]).*").unwrap();
}

#[derive(Debug)]
#[derive(PartialEq)]
pub enum Classification {
    Prefix,
    Center,
    Suffix,
}

#[derive(Debug)]
#[derive(PartialEq)]
pub enum Rule {
    Consonant,
    Vowel,
    Either,
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct Syllable {
    value: String,
    classification: Classification,
    next: Rule,
    previous: Rule,
}

impl Syllable {
    pub fn new(raw: &str) -> Option<Syllable> {
        if FULL_RE.is_match(raw) {
            let (classification, value) = Syllable::classify(raw);
            let syllable = Syllable {
                value,
                classification,
                next: Syllable::determine_next_rule(raw),
                previous: Syllable::determine_previous_rule(raw),
            };
            Some(syllable)
        } else {
            None
        }
    }

    pub fn ends_with_vowel(&self) -> bool {
        VOWELS.contains(&self.value.chars().last().unwrap())
    }

    pub fn starts_with_vowel(&self) -> bool {
        VOWELS.contains(&self.value.chars().next().unwrap())
    }

    fn determine_classification(s: &str) -> Classification {
        match s {
            "-" => Classification::Prefix,
            "+" => Classification::Suffix,
            _   => Classification::Center,
        }
    }

    fn determine_next_rule(s: &str) -> Rule {
        if SUFFIX_RE.is_match(s) {
            Syllable::vowel_or_consonant_flag(s, "+v")
        } else {
            Rule::Either
        }
    }

    fn determine_previous_rule(s: &str) -> Rule {
        if PREFIX_RE.is_match(s) {
            Syllable::vowel_or_consonant_flag(s, "-v")
        } else {
            Rule::Either
        }
    }

    fn vowel_or_consonant_flag(s: &str, matcher: &str) -> Rule {
        if s.to_ascii_lowercase().contains(matcher) {
            Rule::Vowel
        } else {
            Rule::Consonant
        }
    }

    fn classify(raw: &str) -> (Classification, String) {
        let cap = FULL_RE.captures(raw).unwrap();
        return (
            Syllable::determine_classification(&cap[1]),
            cap[2].to_string()
        )
    }
}

mod syllable_tests {
    use super::*;

    #[test]
    fn new_center() {
        let expected = Syllable {
            value: "idr".to_string(),
            classification: Classification::Center,
            next: Rule::Vowel,
            previous: Rule::Consonant,
        };

        let actual = Syllable::new("idr -c +v");

        assert_eq!(expected, actual.unwrap());
    }

    #[test]
    fn new_prefix_any() {
        let expected = Syllable {
            value: "asd".to_string(),
            classification: Classification::Prefix,
            next: Rule::Either,
            previous: Rule::Either,
        };

        let actual = Syllable::new("-asd");

        assert_eq!(expected, actual.unwrap());
    }

    #[test]
    fn new_suffix_any() {
        let expected = Syllable {
            value: "adly".to_string(),
            classification: Classification::Suffix,
            next: Rule::Either,
            previous: Rule::Vowel,
        };

        let actual = Syllable::new("+adly -v");

        assert_eq!(expected, actual.unwrap());
    }

    #[test]
    fn starts_with_vowel() {
        let actual = Syllable::new("+sadly -v");

        assert_eq!(false, actual.unwrap().starts_with_vowel());
    }

    #[test]
    fn starts_with_vowel_not() {
        let actual = Syllable::new("+adly -v");

        assert_eq!(true, actual.unwrap().starts_with_vowel());
    }

    #[test]
    fn ends_with_vowel() {
        let actual = Syllable::new("+sadly -v");

        assert_eq!(true, actual.unwrap().ends_with_vowel());
    }

    #[test]
    fn ends_with_vowel_not() {
        let actual = Syllable::new("-asdf -v");

        assert_eq!(false, actual.unwrap().ends_with_vowel());
    }

    #[test]
    fn determine_classification_prefix() {
        let v = "-";

        assert_eq!(Classification::Prefix, Syllable::determine_classification(v));
    }

    #[test]
    fn determine_classification_center() {
        let v = "";

        assert_eq!(Classification::Center, Syllable::determine_classification(v));
    }

    #[test]
    fn determine_classification_suffix() {
        let v = "+";

        assert_eq!(Classification::Suffix, Syllable::determine_classification(v));
    }

    #[test]
    fn determine_classification_garbage() {
        assert_eq!(Classification::Center, Syllable::determine_classification(" "));
        assert_eq!(Classification::Center, Syllable::determine_classification("asd"));
    }
}

#[cfg(test)]
mod rule_tests {

    use super::*;
    use rstest::rstest;

    #[rstest(input, classification, value,
        case("+sakku -V", Classification::Suffix, "sakku".to_string()),
        case("-darr +v", Classification::Prefix, "darr".to_string()),
        case("drov", Classification::Center, "drov".to_string()),
    )]
    fn classify(input: &str, classification: Classification, value: String) {
        let (actual_classification, actual_value) = Syllable::classify(input);
        assert_eq!(classification, actual_classification);
        assert_eq!(value, actual_value);
    }

    #[rstest(input, expected,
        case("", Rule::Either),
        case("-ahr", Rule::Either),
        case("dus", Rule::Either),
        case("+zou ", Rule::Either),
        case("ez -c +V", Rule::Vowel),
        case("-ahr +v", Rule::Vowel),
        case("-aby +c", Rule::Consonant),
        case("dra +c", Rule::Consonant),
    )]
    fn to_next_rule(input: &str, expected: Rule) {
        assert_eq!(expected, Syllable::determine_next_rule(input));
    }

    #[rstest(input, expected,
        case("", Rule::Either),
        case("-ahr", Rule::Either),
        case("dus", Rule::Either),
        case("+zou ", Rule::Either),
        case("gru -v +c", Rule::Vowel),
        case("+sakku -V", Rule::Vowel),
        case("ay -c", Rule::Consonant),
        case("it -c +v", Rule::Consonant),
    )]
    fn to_previous_rule(input: &str, expected: Rule) {
        assert_eq!(expected, Syllable::determine_previous_rule(input))
    }

    // #[test]
    // fn classify_prefix() {
    //     let (classification, s) = Syllable::classify("-ansr +v".to_string());
    //
    //     assert_eq!(Classification::Prefix, classification);
    //     // assert_eq!("ansr +v".to_string(), s);
    // }


}
