use crate::db::{search_items as search_items_in_db, DbConnection, ItemSummary};
use crate::error::Result;
use crate::state::VaultState;
use rusqlite::Connection;
use tauri::State;

/// 锁定态语义（有意为之，与前端 mock `tauri-mock.ts` 的 `search_items` 保持一致）：
/// 库锁定时本命令返回空列表，而不是返回错误。
///
/// 因此调用方必须用自身的解锁状态区分两种空结果：
/// - 未解锁时：不应调用本命令（前端 store 中有守卫）；
/// - 已解锁但无匹配：才显示「无搜索结果」。
///
/// 匹配字段与规则见 [`crate::db::search_items`]（title / subtitle /
/// username 类字段，LIKE 模糊匹配，通配符按字面量处理）。
pub fn search_items_filtered(
    conn: &Connection,
    vault_state: &VaultState,
    query: &str,
) -> Result<Vec<ItemSummary>> {
    if !vault_state.is_unlocked() {
        return Ok(Vec::new());
    }
    search_items_in_db(conn, query)
}

/// 全库搜索命令：按标题、副标题及用户名类字段模糊匹配。
///
/// - 锁定态：返回空列表（见 [`search_items_filtered`] 的语义说明）。
/// - 空白 query：返回全部条目。
#[tauri::command]
pub fn search_items(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    query: String,
) -> Result<Vec<ItemSummary>> {
    let conn = db
        .0
        .lock()
        .map_err(|_| crate::error::VaultError::DatabaseError("Lock poisoned".to_string()))?;
    search_items_filtered(&conn, &vault_state, &query)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::{derive_key, SecureKey};
    use crate::db::{create_account_item, enable_foreign_keys, init_schema, insert_builtin_groups};

    fn test_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init_schema(&conn).unwrap();
        insert_builtin_groups(&conn).unwrap();
        enable_foreign_keys(&conn).unwrap();
        conn
    }

    fn test_key() -> [u8; 32] {
        derive_key("test-password-1", None).unwrap().key
    }

    fn unlocked_state() -> VaultState {
        let state = VaultState::new();
        state.unlock(SecureKey::new(test_key()));
        state
    }

    /// 两条账号条目：一条常规（标题/用户名都唯一），一条标题带 LIKE 通配符字符。
    fn seed(conn: &Connection, key: &[u8; 32]) -> Vec<String> {
        vec![
            create_account_item(
                conn,
                "built-in-accounts",
                "GitHub",
                "octocat@example.com",
                "pw",
                None,
                None,
                None,
                key,
            )
            .unwrap(),
            create_account_item(conn, "built-in-accounts", "a_b", "literal_user", "pw", None, None, None, key)
                .unwrap(),
        ]
    }

    #[test]
    fn locked_vault_returns_empty_list_not_error() {
        // 锁定态语义：即使库里有数据，也返回空列表且不报错。
        let conn = test_conn();
        let key = test_key();
        seed(&conn, &key);

        let state = VaultState::new(); // 未解锁
        assert!(!state.is_unlocked());

        let results = search_items_filtered(&conn, &state, "").unwrap();
        assert!(results.is_empty(), "locked vault must yield empty results");
    }

    #[test]
    fn empty_query_returns_all_items() {
        let conn = test_conn();
        let key = test_key();
        let ids = seed(&conn, &key);

        // 空字符串与纯空白都视为「无过滤」。
        let empty = search_items_filtered(&conn, &unlocked_state(), "").unwrap();
        let blank = search_items_filtered(&conn, &unlocked_state(), "   ").unwrap();
        assert_eq!(empty.len(), ids.len());
        assert_eq!(blank.len(), ids.len());
    }

    #[test]
    fn matches_title_case_insensitively() {
        let conn = test_conn();
        let key = test_key();
        seed(&conn, &key);

        for query in ["GITHUB", "github", "Git"] {
            let results = search_items_filtered(&conn, &unlocked_state(), query).unwrap();
            assert_eq!(results.len(), 1, "query {query:?} should match only GitHub");
            assert_eq!(results[0].title, "GitHub");
        }
    }

    #[test]
    fn matches_username_fields() {
        let conn = test_conn();
        let key = test_key();
        seed(&conn, &key);

        // "octocat" 只出现在 username 中，不出现在标题里。
        let results = search_items_filtered(&conn, &unlocked_state(), "OCTOCAT").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "GitHub");
    }

    #[test]
    fn like_wildcards_are_treated_literally() {
        let conn = test_conn();
        let key = test_key();
        seed(&conn, &key); // 标题分别为 "GitHub" 与 "a_b"

        // "_" 是字面量：不得把 "a_b" 之外的其他标题当通配符匹配出来。
        let results = search_items_filtered(&conn, &unlocked_state(), "a_b").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "a_b");

        // "%" 也是字面量：无标题包含 "a%b"，结果必须为空。
        let results = search_items_filtered(&conn, &unlocked_state(), "a%b").unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn no_match_returns_empty() {
        let conn = test_conn();
        let key = test_key();
        seed(&conn, &key);

        let results = search_items_filtered(&conn, &unlocked_state(), "zzz-no-such-item").unwrap();
        assert!(results.is_empty());
    }
}
