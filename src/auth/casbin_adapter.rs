//! Casbin [`Adapter`] backed by sqlx [`AnyPool`]（PostgreSQL / MySQL / SQLite）。

use async_trait::async_trait;
use casbin::{Adapter, Filter, Model, Result as CasbinResult};
use sqlx::AnyPool;

use super::casbin_db::{pad_rule_vec, trim_rule};
use crate::entity::{CasbinRulePolicy, SqlBackend};
use crate::error::{AppError, AppResult};

#[derive(Clone)]
pub struct SqlxAnyAdapter {
    pool: AnyPool,
    backend: SqlBackend,
    is_filtered: bool,
}

impl SqlxAnyAdapter {
    pub fn new(pool: AnyPool, backend: SqlBackend) -> Self {
        Self {
            pool,
            backend,
            is_filtered: false,
        }
    }

    fn map_err(e: sqlx::Error) -> casbin::Error {
        casbin::Error::AdapterError(casbin::error::AdapterError(Box::new(e)))
    }

    fn map_app_err(e: AppError) -> casbin::Error {
        casbin::Error::AdapterError(casbin::error::AdapterError(Box::new(
            std::io::Error::new(std::io::ErrorKind::Other, e.to_string()),
        )))
    }
}

#[async_trait]
impl Adapter for SqlxAnyAdapter {
    async fn load_policy(&mut self, m: &mut dyn Model) -> CasbinResult<()> {
        let rows = sqlx::query_as::<_, CasbinRulePolicy>(CasbinRulePolicy::select_all_ordered())
            .fetch_all(&self.pool)
            .await
            .map_err(Self::map_err)?;

        for row in rows {
            let ptype = row.ptype.clone();
            let rule = trim_rule(row.into_rule_values());
            insert_model_rule(m, &ptype, rule)?;
        }
        Ok(())
    }

    async fn load_filtered_policy<'a>(
        &mut self,
        m: &mut dyn Model,
        f: Filter<'a>,
    ) -> CasbinResult<()> {
        let rows = sqlx::query_as::<_, CasbinRulePolicy>(CasbinRulePolicy::select_all_ordered())
            .fetch_all(&self.pool)
            .await
            .map_err(Self::map_err)?;

        for row in rows {
            let ptype = row.ptype.clone();
            let rule = trim_rule(row.into_rule_values());
            let sec = section_for_ptype(&ptype);
            let mut skip = false;

            if sec == "p" {
                for (i, expected) in f.p.iter().enumerate() {
                    if !expected.is_empty() && rule.get(i).map(String::as_str) != Some(*expected) {
                        skip = true;
                    }
                }
            }
            if sec == "g" {
                for (i, expected) in f.g.iter().enumerate() {
                    if !expected.is_empty() && rule.get(i).map(String::as_str) != Some(*expected) {
                        skip = true;
                    }
                }
            }

            if skip {
                self.is_filtered = true;
                continue;
            }
            insert_model_rule(m, &ptype, rule)?;
        }
        Ok(())
    }

    async fn save_policy(&mut self, m: &mut dyn Model) -> CasbinResult<()> {
        sqlx::query("DELETE FROM casbin_rule")
            .execute(&self.pool)
            .await
            .map_err(Self::map_err)?;

        if let Some(ast_map) = m.get_model().get("p") {
            for (ptype, ast) in ast_map {
                for rule in ast.get_policy() {
                    insert_rule_db(&self.pool, self.backend, ptype, rule)
                        .await
                        .map_err(Self::map_app_err)?;
                }
            }
        }
        if let Some(ast_map) = m.get_model().get("g") {
            for (ptype, ast) in ast_map {
                for rule in ast.get_policy() {
                    insert_rule_db(&self.pool, self.backend, ptype, rule)
                        .await
                        .map_err(Self::map_app_err)?;
                }
            }
        }
        Ok(())
    }

    async fn clear_policy(&mut self) -> CasbinResult<()> {
        sqlx::query("DELETE FROM casbin_rule")
            .execute(&self.pool)
            .await
            .map_err(Self::map_err)?;
        self.is_filtered = false;
        Ok(())
    }

    async fn add_policy(
        &mut self,
        _sec: &str,
        ptype: &str,
        rule: Vec<String>,
    ) -> CasbinResult<bool> {
        let inserted = insert_rule_db(&self.pool, self.backend, ptype, &rule)
            .await
            .map_err(Self::map_app_err)?;
        Ok(inserted)
    }

    async fn add_policies(
        &mut self,
        sec: &str,
        ptype: &str,
        rules: Vec<Vec<String>>,
    ) -> CasbinResult<bool> {
        let mut any = false;
        for rule in rules {
            if self.add_policy(sec, ptype, rule).await? {
                any = true;
            }
        }
        Ok(any)
    }

    async fn remove_policy(
        &mut self,
        _sec: &str,
        ptype: &str,
        rule: Vec<String>,
    ) -> CasbinResult<bool> {
        let cols = pad_rule_vec(&rule);
        let result = sqlx::query(
            r#"
            DELETE FROM casbin_rule
            WHERE ptype = ? AND v0 = ? AND v1 = ? AND v2 = ? AND v3 = ? AND v4 = ? AND v5 = ?
            "#,
        )
        .bind(ptype)
        .bind(&cols[0])
        .bind(&cols[1])
        .bind(&cols[2])
        .bind(&cols[3])
        .bind(&cols[4])
        .bind(&cols[5])
        .execute(&self.pool)
        .await
        .map_err(Self::map_err)?;
        Ok(result.rows_affected() > 0)
    }

    async fn remove_policies(
        &mut self,
        sec: &str,
        ptype: &str,
        rules: Vec<Vec<String>>,
    ) -> CasbinResult<bool> {
        let mut any = false;
        for rule in rules {
            if self.remove_policy(sec, ptype, rule).await? {
                any = true;
            }
        }
        Ok(any)
    }

    async fn remove_filtered_policy(
        &mut self,
        _sec: &str,
        ptype: &str,
        field_index: usize,
        field_values: Vec<String>,
    ) -> CasbinResult<bool> {
        let mut sql = String::from("DELETE FROM casbin_rule WHERE ptype = ?");
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
        let result = query.execute(&self.pool).await.map_err(Self::map_err)?;
        Ok(result.rows_affected() > 0)
    }

    fn is_filtered(&self) -> bool {
        self.is_filtered
    }
}

fn section_for_ptype(ptype: &str) -> &str {
    match ptype.chars().next() {
        Some('p') => "p",
        Some('g') => "g",
        _ => "p",
    }
}

fn insert_model_rule(m: &mut dyn Model, ptype: &str, rule: Vec<String>) -> CasbinResult<()> {
    let sec = section_for_ptype(ptype);
    if let Some(ast_map) = m.get_mut_model().get_mut(sec) {
        if let Some(ast) = ast_map.get_mut(ptype) {
            ast.policy.insert(rule);
        }
    }
    Ok(())
}

async fn insert_rule_db(
    pool: &AnyPool,
    backend: SqlBackend,
    ptype: &str,
    rule: &[String],
) -> AppResult<bool> {
    let cols = pad_rule_vec(rule);
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
        .map_err(|e| AppError::Internal(format!("casbin insert: {e}")))?;
    Ok(result.rows_affected() > 0)
}
