//! 与 `casbin_rule` 表对应的实体（Casbin 策略持久化）。

/// 表名。
pub const TABLE: &str = "casbin_rule";

/// Casbin 策略完整行（含自增主键；DDL 对应，当前查询只用 [`CasbinRulePolicy`]）。
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct CasbinRule {
    pub id: i64,
    pub ptype: String,
    pub v0: String,
    pub v1: String,
    pub v2: String,
    pub v3: String,
    pub v4: String,
    pub v5: String,
}

/// 加载策略时使用的列投影（不含 `id`）。
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct CasbinRulePolicy {
    pub ptype: String,
    pub v0: String,
    pub v1: String,
    pub v2: String,
    pub v3: String,
    pub v4: String,
    pub v5: String,
}

impl CasbinRulePolicy {
    pub fn into_rule_values(self) -> [String; 6] {
        [self.v0, self.v1, self.v2, self.v3, self.v4, self.v5]
    }
}
