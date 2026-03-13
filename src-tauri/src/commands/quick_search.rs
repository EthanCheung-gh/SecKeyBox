use crate::db::{get_all_items, DbConnection, ItemSummary};
use crate::error::Result;
use crate::state::VaultState;
use tauri::State;

#[tauri::command]
pub fn search_items(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    query: String,
) -> Result<Vec<ItemSummary>> {
    if !vault_state.is_unlocked() {
        return Ok(vec![]);
    }

    let conn =
        db.0.lock()
            .map_err(|_| crate::error::VaultError::DatabaseError("Lock poisoned".to_string()))?;

    let all_items = get_all_items(&conn)?;

    if query.is_empty() {
        return Ok(all_items);
    }

    let query_lower = query.to_lowercase();
    let filtered: Vec<ItemSummary> = all_items
        .into_iter()
        .filter(|item| {
            item.title.to_lowercase().contains(&query_lower)
                || item.subtitle.to_lowercase().contains(&query_lower)
        })
        .collect();

    Ok(filtered)
}
