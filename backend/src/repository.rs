use crate::models::*;
use anyhow::Result;
use sqlx::SqlitePool;

#[derive(Debug, Clone)]
pub struct Repository {
    pool: SqlitePool,
}

impl Repository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    // User operations
    pub async fn get_or_create_user(&self, username: &str) -> Result<User> {
        // Try to get existing user
        let user = sqlx::query_as::<_, User>(
            "SELECT id, username, password_hash, created_at, updated_at FROM users WHERE username = ?",
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(user) = user {
            return Ok(user);
        }

        // Create new user (without password for Remote-User auth)
        let result = sqlx::query("INSERT INTO users (username) VALUES (?)")
            .bind(username)
            .execute(&self.pool)
            .await?;

        let user = sqlx::query_as::<_, User>(
            "SELECT id, username, password_hash, created_at, updated_at FROM users WHERE id = ?",
        )
        .bind(result.last_insert_rowid())
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, username, password_hash, created_at, updated_at FROM users WHERE username = ?",
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn get_user_by_id(&self, user_id: i64) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, username, password_hash, created_at, updated_at FROM users WHERE id = ?",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn create_user(&self, username: &str, password_hash: &str) -> Result<User> {
        let result = sqlx::query("INSERT INTO users (username, password_hash) VALUES (?, ?)")
            .bind(username)
            .bind(password_hash)
            .execute(&self.pool)
            .await?;

        let user = sqlx::query_as::<_, User>(
            "SELECT id, username, password_hash, created_at, updated_at FROM users WHERE id = ?",
        )
        .bind(result.last_insert_rowid())
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    // LLM Platform operations
    pub async fn create_llm_platform(
        &self,
        user_id: i64,
        req: CreateLlmPlatformRequest,
    ) -> Result<LlmPlatform> {
        let result = sqlx::query(
            "INSERT INTO llm_platforms (user_id, name, base_url, api_key, platform_type) VALUES (?, ?, ?, ?, ?)"
        )
        .bind(user_id)
        .bind(&req.name)
        .bind(&req.base_url)
        .bind(&req.api_key)
        .bind(&req.platform_type)
        .execute(&self.pool)
        .await?;

        let platform = sqlx::query_as::<_, LlmPlatform>("SELECT * FROM llm_platforms WHERE id = ?")
            .bind(result.last_insert_rowid())
            .fetch_one(&self.pool)
            .await?;

        Ok(platform)
    }

    pub async fn get_llm_platforms(&self, user_id: i64) -> Result<Vec<LlmPlatform>> {
        let platforms = sqlx::query_as::<_, LlmPlatform>(
            "SELECT * FROM llm_platforms WHERE user_id = ? ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(platforms)
    }

    pub async fn get_llm_platform(&self, id: i64, user_id: i64) -> Result<Option<LlmPlatform>> {
        let platform = sqlx::query_as::<_, LlmPlatform>(
            "SELECT * FROM llm_platforms WHERE id = ? AND user_id = ?",
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(platform)
    }

    pub async fn update_llm_platform(
        &self,
        id: i64,
        user_id: i64,
        req: UpdateLlmPlatformRequest,
    ) -> Result<Option<LlmPlatform>> {
        // Build query based on whether api_key is provided
        let result = if let Some(api_key) = &req.api_key {
            // Update including api_key
            sqlx::query(
                r#"UPDATE llm_platforms 
                   SET name = ?, base_url = ?, api_key = ?, platform_type = ?, updated_at = CURRENT_TIMESTAMP
                   WHERE id = ? AND user_id = ?"#,
            )
            .bind(&req.name)
            .bind(&req.base_url)
            .bind(api_key)
            .bind(&req.platform_type)
            .bind(id)
            .bind(user_id)
            .execute(&self.pool)
            .await?
        } else {
            // Update without changing api_key
            sqlx::query(
                r#"UPDATE llm_platforms 
                   SET name = ?, base_url = ?, platform_type = ?, updated_at = CURRENT_TIMESTAMP
                   WHERE id = ? AND user_id = ?"#,
            )
            .bind(&req.name)
            .bind(&req.base_url)
            .bind(&req.platform_type)
            .bind(id)
            .bind(user_id)
            .execute(&self.pool)
            .await?
        };

        if result.rows_affected() == 0 {
            return Ok(None);
        }

        let platform = sqlx::query_as::<_, LlmPlatform>("SELECT * FROM llm_platforms WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;

        Ok(Some(platform))
    }

    pub async fn delete_llm_platform(&self, id: i64, user_id: i64) -> Result<bool> {
        let result = sqlx::query("DELETE FROM llm_platforms WHERE id = ? AND user_id = ?")
            .bind(id)
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    // Proxy API Key operations
    pub async fn create_proxy_api_key(
        &self,
        user_id: i64,
        name: &str,
        key_hash: &str,
        key_prefix: &str,
        llm_platform_id: i64,
    ) -> Result<ProxyApiKey> {
        let result = sqlx::query(
            "INSERT INTO proxy_api_keys (user_id, name, key_hash, key_prefix, llm_platform_id) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(user_id)
        .bind(name)
        .bind(key_hash)
        .bind(key_prefix)
        .bind(llm_platform_id)
        .execute(&self.pool)
        .await?;

        let key = sqlx::query_as::<_, ProxyApiKey>("SELECT * FROM proxy_api_keys WHERE id = ?")
            .bind(result.last_insert_rowid())
            .fetch_one(&self.pool)
            .await?;

        Ok(key)
    }

    pub async fn get_proxy_api_keys(&self, user_id: i64) -> Result<Vec<ProxyApiKey>> {
        let keys = sqlx::query_as::<_, ProxyApiKey>(
            "SELECT * FROM proxy_api_keys WHERE user_id = ? ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(keys)
    }

    pub async fn get_proxy_api_key_by_hash(&self, key_hash: &str) -> Result<Option<ProxyApiKey>> {
        let key =
            sqlx::query_as::<_, ProxyApiKey>("SELECT * FROM proxy_api_keys WHERE key_hash = ?")
                .bind(key_hash)
                .fetch_optional(&self.pool)
                .await?;

        Ok(key)
    }

    pub async fn update_proxy_api_key_last_used(&self, id: i64) -> Result<()> {
        sqlx::query("UPDATE proxy_api_keys SET last_used_at = datetime('now') WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn delete_proxy_api_key(&self, id: i64, user_id: i64) -> Result<bool> {
        let result = sqlx::query("DELETE FROM proxy_api_keys WHERE id = ? AND user_id = ?")
            .bind(id)
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    // Request Log operations
    pub async fn create_request_log(&self, req: CreateRequestLogRequest) -> Result<RequestLog> {
        let result = sqlx::query(
            r#"INSERT INTO request_logs 
            (user_id, proxy_api_key_id, llm_platform_id, method, path, request_headers, request_body, 
             outgoing_url, outgoing_headers, outgoing_body,
             response_status, response_headers, response_body, duration_ms, error)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#
        )
        .bind(req.user_id)
        .bind(req.proxy_api_key_id)
        .bind(req.llm_platform_id)
        .bind(&req.method)
        .bind(&req.path)
        .bind(&req.request_headers)
        .bind(&req.request_body)
        .bind(&req.outgoing_url)
        .bind(&req.outgoing_headers)
        .bind(&req.outgoing_body)
        .bind(req.response_status)
        .bind(&req.response_headers)
        .bind(&req.response_body)
        .bind(req.duration_ms)
        .bind(&req.error)
        .execute(&self.pool)
        .await?;

        let log = sqlx::query_as::<_, RequestLog>("SELECT * FROM request_logs WHERE id = ?")
            .bind(result.last_insert_rowid())
            .fetch_one(&self.pool)
            .await?;

        Ok(log)
    }

    pub async fn get_request_logs(&self, user_id: i64, limit: i64) -> Result<Vec<RequestLog>> {
        let logs = sqlx::query_as::<_, RequestLog>(
            "SELECT * FROM request_logs WHERE user_id = ? ORDER BY created_at DESC LIMIT ?",
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(logs)
    }

    pub async fn get_request_log(&self, id: i64, user_id: i64) -> Result<Option<RequestLog>> {
        let log = sqlx::query_as::<_, RequestLog>(
            "SELECT * FROM request_logs WHERE id = ? AND user_id = ?",
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(log)
    }
}
