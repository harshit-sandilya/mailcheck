use crate::models::company::CompanyRegistry;
use crate::models::config::Config;
use anyhow::Result;

pub trait Store {
    fn load_config(&self) -> Result<Config>;
    fn get_email(&self) -> Result<String>;
    fn get_delay(&self) -> Result<i64>;
    fn set_email(&self, value: String) -> Result<()>;
    fn set_delay(&self, value: i64) -> Result<()>;

    fn get_patterns(&self) -> Result<Vec<String>>;
    fn add_pattern(&self, pattern: String) -> Result<()>;
    fn remove_pattern(&self, pattern: String) -> Result<()>;
    fn reset_patterns(&self) -> Result<()>;

    fn load_local_companies(&self) -> Result<CompanyRegistry>;
    fn upsert_company_pattern(
        &self,
        domain: String,
        pattern: String,
        confidence: u8,
        samples: u32,
    ) -> Result<()>;
    fn reset_company(&self, domain: String) -> Result<bool>;
}
