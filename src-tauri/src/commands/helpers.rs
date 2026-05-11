use std::{collections::HashSet};
use rand::{distr::Alphanumeric, distr::SampleString, rng};

/************************************************************************************************************************\

HELPER FUNCTIONS

\************************************************************************************************************************/

pub fn validate_password(pw: &str) -> bool {
    let has_min_length = pw.chars().count() >= 10;
    let no_spaces = !pw.chars().any(|c| c.is_whitespace());
    let has_numbers = pw.chars().any(|c| c.is_ascii_digit());
    let has_uppercase = pw.chars().any(|c| c.is_uppercase() && c.is_alphabetic());
    let has_lowercase = pw.chars().any(|c| c.is_lowercase() && c.is_alphabetic());
    let has_special_char = pw.chars().any(|c| r#"=_!@#$€£¤%^&*(){}[],.?'":|/\+-<>~§"#.contains(c));

    has_min_length && no_spaces && has_numbers && has_uppercase && has_lowercase && has_special_char
}

pub fn generate_recovery_key () -> String {
    Alphanumeric.sample_string(&mut rng(), 48)
}

pub fn valid_categories () -> HashSet<&'static str> {
    HashSet::from([
        "rent", "taxes", "groceries", "utilities", "transportation", "travel", "entertainment", "healthcare",
        "insurance", "subscription", "education", "other", "salary", "freelance", "investments",
    ])
}

pub fn valid_transaction_types () -> HashSet<&'static str> {
    HashSet::from(["income", "expense"])
}