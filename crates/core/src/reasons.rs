//! Reasons for an absence: each one valid or not valid. Names are unique without case and outer
//! spaces.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReasonId(pub i64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ReasonError {
    #[error("a name is needed")]
    NameRequired,
    #[error("there is already a reason with this name")]
    NameTaken,
}

/// The name without outer spaces, checked against `others`, the names of the other reasons
/// (without the reason that is renamed).
pub fn clean_reason_name(name: &str, others: &[&str]) -> Result<String, ReasonError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(ReasonError::NameRequired);
    }
    let key = name.to_lowercase();
    if others
        .iter()
        .any(|other| other.trim().to_lowercase() == key)
    {
        return Err(ReasonError::NameTaken);
    }
    Ok(name.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_name_is_trimmed() {
        assert_eq!(
            clean_reason_name(" Family ", &["School"]),
            Ok("Family".to_owned())
        );
    }

    #[test]
    fn a_name_is_needed() {
        assert_eq!(clean_reason_name("  ", &[]), Err(ReasonError::NameRequired));
    }

    #[test]
    fn names_are_unique_without_case_and_outer_spaces() {
        assert_eq!(
            clean_reason_name("school", &["School", "Ill"]),
            Err(ReasonError::NameTaken)
        );
        assert_eq!(
            clean_reason_name("Ill", &[" ill "]),
            Err(ReasonError::NameTaken)
        );
    }

    #[test]
    fn a_reason_can_get_its_own_name_with_another_case() {
        // The renamed reason is not in `others`.
        assert_eq!(
            clean_reason_name("work", &["School"]),
            Ok("work".to_owned())
        );
    }
}
