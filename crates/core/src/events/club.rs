//! Clubs from the source's club list, and the search by name.

/// The source's own club number (not the federation's number).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ClubNumber(pub i64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Club {
    pub number: ClubNumber,
    pub name: String,
    /// Some clubs have no town in the source.
    pub town: Option<String>,
}

/// The clubs whose name contains the query, upper or lower case not important, sorted by name.
/// An empty query finds nothing.
pub fn search_clubs(clubs: &[Club], query: &str) -> Vec<Club> {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return Vec::new();
    }
    let mut found: Vec<Club> = clubs
        .iter()
        .filter(|club| club.name.to_lowercase().contains(&query))
        .cloned()
        .collect();
    found.sort_by_cached_key(|club| (club.name.to_lowercase(), club.number));
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    fn club(number: i64, name: &str, town: Option<&str>) -> Club {
        Club {
            number: ClubNumber(number),
            name: name.to_owned(),
            town: town.map(str::to_owned),
        }
    }

    fn clubs() -> Vec<Club> {
        vec![
            club(37, "Blue Drakes", Some("Woerden")),
            club(21, "Green Giants", Some("Leusden")),
            club(5, "Drakes Delta", None),
            club(24, "Utrecht Basketball", Some("Utrecht")),
        ]
    }

    fn names(found: &[Club]) -> Vec<&str> {
        found.iter().map(|club| club.name.as_str()).collect()
    }

    #[test]
    fn finds_a_club_by_part_of_its_name() {
        let found = search_clubs(&clubs(), "giants");
        assert_eq!(found, vec![club(21, "Green Giants", Some("Leusden"))]);
    }

    #[test]
    fn upper_or_lower_case_and_spaces_around_do_not_matter() {
        for query in ["drakes", "DRAKES", " drakes "] {
            assert_eq!(
                names(&search_clubs(&clubs(), query)),
                ["Blue Drakes", "Drakes Delta"],
                "query {query:?}"
            );
        }
    }

    #[test]
    fn no_match_finds_nothing() {
        assert!(search_clubs(&clubs(), "zzz").is_empty());
    }

    #[test]
    fn an_empty_query_finds_nothing() {
        assert!(search_clubs(&clubs(), "").is_empty());
        assert!(search_clubs(&clubs(), "   ").is_empty());
    }

    #[test]
    fn results_are_sorted_by_name() {
        assert_eq!(
            names(&search_clubs(&clubs(), "e")),
            [
                "Blue Drakes",
                "Drakes Delta",
                "Green Giants",
                "Utrecht Basketball"
            ]
        );
    }
}
