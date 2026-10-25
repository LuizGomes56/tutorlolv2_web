/// Match the existing ASCII-only search semantics without temporary strings.
pub fn contains_ascii_case_insensitive(name: &str, query: &str) -> bool {
    query.is_empty()
        || name
            .as_bytes()
            .windows(query.len())
            .any(|part| part.eq_ignore_ascii_case(query.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::contains_ascii_case_insensitive;

    #[test]
    fn search_preserves_ascii_folding_empty_queries_and_non_ascii_bytes() {
        for (name, query, expected) in [
            ("Nashor's Tooth", "TOOTH", true),
            ("Kai'Sa", "kai's", true),
            ("Nashor's Tooth", "", true),
            ("", "", true),
            ("", "item", false),
            ("Armor", "armor penetration", false),
            ("Kai'Sa", " kai", false),
            ("Élan", "ÉLAN", true),
            ("Élan", "élan", false),
            ("界", "界", true),
            ("界", "世", false),
        ] {
            assert_eq!(
                contains_ascii_case_insensitive(name, query),
                expected,
                "{name:?}, {query:?}"
            );
        }
    }
}
