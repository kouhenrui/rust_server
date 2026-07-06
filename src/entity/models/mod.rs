//! 数据库实体 struct 定义（与 `schema.rs` 中的 DDL 一一对应）。

pub(crate) mod account;
pub(crate) mod casbin_rule;

pub use account::{Account, AccountAuth};
pub use casbin_rule::CasbinRulePolicy;

/// 表名常量（供 DDL / 查询引用）。
pub mod tables {
    pub use super::account::TABLE as ACCOUNTS;
    pub use super::casbin_rule::TABLE as CASBIN_RULE;
}
