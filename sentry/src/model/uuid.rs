use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash, sqlx::Type, Default)]
#[sqlx(transparent)]
pub struct UUID(pub String);
