//! 实体表的数据访问层（CRUD / 查询）。

pub(crate) mod account;
pub(crate) mod casbin_rule;

pub use account::AccountRepository;
pub use casbin_rule::CasbinRuleRepository;

pub(crate) fn sql_err(table: &str, op: &str, e: impl std::fmt::Display) -> crate::error::AppError {
    crate::error::AppError::Internal(format!("{table} {op}: {e}"))
}
