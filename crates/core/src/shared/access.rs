//! Who may use the tool (decision 0007).

/// A logged-in person, as the login server describes them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identity {
    /// The login server's stable user id.
    pub subject: String,
    pub name: String,
    pub groups: Vec<String>,
}

/// True when the identity has the access role. The match is exact and case-sensitive.
pub fn may_access(identity: &Identity, role: &str) -> bool {
    identity.groups.iter().any(|group| group == role)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity(groups: &[&str]) -> Identity {
        Identity {
            subject: "subject-1".to_owned(),
            name: "Coordinator".to_owned(),
            groups: groups.iter().map(|group| (*group).to_owned()).collect(),
        }
    }

    #[test]
    fn allows_the_access_role() {
        assert!(may_access(
            &identity(&["family", "team_coordinator"]),
            "team_coordinator"
        ));
    }

    #[test]
    fn refuses_no_groups() {
        assert!(!may_access(&identity(&[]), "team_coordinator"));
    }

    #[test]
    fn refuses_other_groups() {
        assert!(!may_access(
            &identity(&["family", "admins"]),
            "team_coordinator"
        ));
    }

    #[test]
    fn match_is_case_sensitive() {
        assert!(!may_access(
            &identity(&["Team_Coordinator"]),
            "team_coordinator"
        ));
    }
}
