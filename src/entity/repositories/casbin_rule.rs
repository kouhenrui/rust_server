//! `casbin_rule` 表仓储。

use crate::entity::models::casbin_rule::{CasbinRulePolicy, TABLE};
use crate::entity::repositories::sql_err;
use crate::entity::SqlBackend;
use crate::error::AppResult;
use sqlx::AnyPool;

const SELECT_ALL_ORDERED: &str =
    "SELECT ptype, v0, v1, v2, v3, v4, v5 FROM casbin_rule ORDER BY id";

const DELETE_ALL: &str = "DELETE FROM casbin_rule";

const DELETE_BY_RULE: &str = "DELETE FROM casbin_rule \
     WHERE ptype = ? AND v0 = ? AND v1 = ? AND v2 = ? AND v3 = ? AND v4 = ? AND v5 = ?";

/// `casbin_rule` 表访问入口。
pub struct CasbinRuleRepository;

impl CasbinRuleRepository {
    pub async fn count(pool: &AnyPool) -> AppResult<i64> {
        let (count,): (i64,) = sqlx::query_as(&format!("SELECT COUNT(*) FROM {TABLE}"))
            .fetch_one(pool)
            .await
            .map_err(|e| sql_err(TABLE, "count", e))?;
        Ok(count)
    }

    pub async fn list_all_ordered(pool: &AnyPool) -> AppResult<Vec<CasbinRulePolicy>> {
        sqlx::query_as(SELECT_ALL_ORDERED)
            .fetch_all(pool)
            .await
            .map_err(|e| sql_err(TABLE, "list", e))
    }

    pub async fn insert(
        pool: &AnyPool,
        backend: SqlBackend,
        ptype: &str,
        rule: &[String],
    ) -> AppResult<bool> {
        let cols = pad_rule(rule);
        let result = sqlx::query(backend.casbin_insert_sql())
            .bind(ptype)
            .bind(&cols[0])
            .bind(&cols[1])
            .bind(&cols[2])
            .bind(&cols[3])
            .bind(&cols[4])
            .bind(&cols[5])
            .execute(pool)
            .await
            .map_err(|e| sql_err(TABLE, "insert", e))?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn insert_slices(
        pool: &AnyPool,
        backend: SqlBackend,
        ptype: &str,
        rule: &[&str],
    ) -> AppResult<bool> {
        let owned: Vec<String> = rule.iter().map(|s| (*s).to_string()).collect();
        Self::insert(pool, backend, ptype, &owned).await
    }

    pub async fn delete_all(pool: &AnyPool) -> AppResult<()> {
        sqlx::query(DELETE_ALL)
            .execute(pool)
            .await
            .map_err(|e| sql_err(TABLE, "delete_all", e))?;
        Ok(())
    }

    pub async fn delete_by_rule(pool: &AnyPool, ptype: &str, rule: &[String]) -> AppResult<bool> {
        let cols = pad_rule(rule);
        let result = sqlx::query(DELETE_BY_RULE)
            .bind(ptype)
            .bind(&cols[0])
            .bind(&cols[1])
            .bind(&cols[2])
            .bind(&cols[3])
            .bind(&cols[4])
            .bind(&cols[5])
            .execute(pool)
            .await
            .map_err(|e| sql_err(TABLE, "delete_by_rule", e))?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn delete_filtered(
        pool: &AnyPool,
        ptype: &str,
        field_index: usize,
        field_values: &[String],
    ) -> AppResult<bool> {
        let mut sql = format!("DELETE FROM {TABLE} WHERE ptype = ?");
        let mut binds: Vec<String> = vec![ptype.to_string()];
        let columns = ["v0", "v1", "v2", "v3", "v4", "v5"];

        if field_index <= columns.len() {
            for (offset, value) in field_values.iter().enumerate() {
                if value.is_empty() {
                    continue;
                }
                let col = columns[field_index + offset];
                sql.push_str(&format!(" AND {col} = ?"));
                binds.push(value.clone());
            }
        }

        let mut query = sqlx::query(&sql);
        for value in &binds {
            query = query.bind(value);
        }
        let result = query
            .execute(pool)
            .await
            .map_err(|e| sql_err(TABLE, "delete_filtered", e))?;
        Ok(result.rows_affected() > 0)
    }
}

pub(crate) fn pad_rule(rule: &[String]) -> [String; 6] {
    let mut cols = std::array::from_fn(|_| String::new());
    for (i, value) in rule.iter().enumerate().take(6) {
        cols[i] = value.clone();
    }
    cols
}

pub fn trim_rule(cols: [String; 6]) -> Vec<String> {
    let mut rule = cols.to_vec();
    while rule.last().is_some_and(|s| s.is_empty()) {
        rule.pop();
    }
    rule
}
