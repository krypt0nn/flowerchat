use std::str::FromStr;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Role {
    #[default]
    User,
    Moderator,
    Administrator,
    Owner
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::User          => f.write_str("user"),
            Self::Moderator     => f.write_str("moderator"),
            Self::Administrator => f.write_str("administrator"),
            Self::Owner         => f.write_str("owner")
        }
    }
}

impl FromStr for Role {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "user"          | "member"             => Ok(Self::User),
            "moderator"     | "mod"     | "moder"  => Ok(Self::Moderator),
            "administrator" | "admin"              => Ok(Self::Administrator),
            "owner"         | "creator" | "author" => Ok(Self::Owner),

            _ => Err(s.to_string())
        }
    }
}

#[test]
fn test_roles() {
    const ROLES: &[Role] = &[
        Role::User,
        Role::Moderator,
        Role::Administrator,
        Role::Owner
    ];

    for i in 1..ROLES.len() {
        assert!(ROLES[i] > ROLES[i - 1]);
    }

    for role in ROLES {
        assert_eq!(Role::from_str(&role.to_string()), Ok(*role));
    }
}
