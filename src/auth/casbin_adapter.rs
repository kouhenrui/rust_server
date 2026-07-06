//! Casbin [`Adapter`] backed by [`CasbinRuleRepository`].

use async_trait::async_trait;
use casbin::{Adapter, Filter, Model, Result as CasbinResult};

use crate::entity::trim_rule;
use crate::entity::{CasbinRuleRepository, SqlBackend};
use sqlx::AnyPool;

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

    fn map_err(e: crate::error::AppError) -> casbin::Error {
        casbin::Error::AdapterError(casbin::error::AdapterError(Box::new(
            std::io::Error::other(e.to_string()),
        )))
    }
}

#[async_trait]
impl Adapter for SqlxAnyAdapter {
    async fn load_policy(&mut self, m: &mut dyn Model) -> CasbinResult<()> {
        let rows = CasbinRuleRepository::list_all_ordered(&self.pool)
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
        let rows = CasbinRuleRepository::list_all_ordered(&self.pool)
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
        CasbinRuleRepository::delete_all(&self.pool)
            .await
            .map_err(Self::map_err)?;

        if let Some(ast_map) = m.get_model().get("p") {
            for (ptype, ast) in ast_map {
                for rule in ast.get_policy() {
                    CasbinRuleRepository::insert(&self.pool, self.backend, ptype, rule)
                        .await
                        .map_err(Self::map_err)?;
                }
            }
        }
        if let Some(ast_map) = m.get_model().get("g") {
            for (ptype, ast) in ast_map {
                for rule in ast.get_policy() {
                    CasbinRuleRepository::insert(&self.pool, self.backend, ptype, rule)
                        .await
                        .map_err(Self::map_err)?;
                }
            }
        }
        Ok(())
    }

    async fn clear_policy(&mut self) -> CasbinResult<()> {
        CasbinRuleRepository::delete_all(&self.pool)
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
        CasbinRuleRepository::insert(&self.pool, self.backend, ptype, &rule)
            .await
            .map_err(Self::map_err)
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
        CasbinRuleRepository::delete_by_rule(&self.pool, ptype, &rule)
            .await
            .map_err(Self::map_err)
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
        CasbinRuleRepository::delete_filtered(&self.pool, ptype, field_index, &field_values)
            .await
            .map_err(Self::map_err)
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
