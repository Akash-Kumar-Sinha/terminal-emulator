// INFO: Icon and action label representing the Git Commit action.
pub struct CommitIcon;

impl CommitIcon {
    pub const LABEL: &'static str = "Commit";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commit_icon() {
        assert_eq!(CommitIcon::LABEL, "Commit");
    }
}
