use sqlx::{QueryBuilder, query_as};
use tauri::State;
use log::{info, error, warn};
use ammonia;
use std::collections::{HashSet, HashMap};

use crate::{
    AppState, models::{
        note::{Note, NoteUpdate, StoredNote},
        tab::{Tab, TabIdTitle}
    }, services::{
        cache::{CacheData, UpdateTask},
        crypto::{KEY_LEN, decrypt, encrypt},
        session::SessionData,
    }, utils::time::create_timestamp,
};

async fn fetch_and_cache_notes(
    state: &AppState,
    key: &str,
    user_id: i64,
    username: &str,
    dek: &[u8; KEY_LEN],
) -> Result<Vec<Note>, String> {
    let stored = query_as::<_, StoredNote>("SELECT * FROM notes WHERE user_id = ?")
        .bind(user_id)
        .fetch_all(&state.db)
        .await
        .map_err(|e| {
            error!("Failed to fetch notes for user '{}': {:#?}", username, e);
            "Database error".to_string()
        })?;

    let mut notes = Vec::<Note>::new();
    for note in stored {
        let title = String::from_utf8(decrypt(dek, &note.title, "notes.title", user_id, note.id)?).map_err(|e| {
            error!("Failed to parse note title during fetch: {:#?}", e);
            "An error occurred".to_string()
        })?;
        let content = String::from_utf8(decrypt(dek, &note.content, "notes.content", user_id, note.id)?).map_err(|e| {
            error!("Failed to parse note content during fetch: {:#?}", e);
            "An error occurred".to_string()
        })?;

        notes.push(Note {
            id: note.id,
            user_id: note.user_id,
            tab_id: note.tab_id,
            order_id: note.order_id,
            title,
            content,
            bg_color: note.bg_color,
        });
    }
    notes.sort_by_key(|n| n.order_id);
    state.session.cache.cache_results(key.to_string(), CacheData::from(notes.clone()), "Failed to set notes to cache during fetch");

    Ok(notes)
}

#[tauri::command]
pub async fn create_note(
    state: State<'_, AppState>,
    tab_id: i64,
    title: String,
    content: String,
) -> Result<Note, String> {
    let state: &AppState = &state;

    let session: SessionData = state.session.get_session().map_err(|e| {
        error!("Creating note failed at {} due to: {:#?}", create_timestamp(), e);
        "An error occurred".to_string()
    })?;

    if tab_id.le(&0) {
        error!("User '{}' tried creating a note with an invalid tab ID: '{}'", session.user.name, tab_id);
        return Err("An error occurred".to_string());
    }

    let mut tx = state.db.begin().await.map_err(|e| {
        error!("Failed to begin transaction: {:#?}", e);
        "Database error".to_string()
    })?;

    let max_order_id = sqlx::query_scalar::<_, u32>("SELECT MAX(order_id) FROM notes WHERE tab_id = ? AND user_id = ?")
        .bind(tab_id)
        .bind(session.user.id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| {
            error!("Failed to fetch the highest current order ID for creating note: {:#?}", e);
            "Database error".to_string()
        })?
        .unwrap_or(0);

    let new_order_id = max_order_id + 1;

    let mut cleaner = ammonia::Builder::new();
    cleaner
        .add_tags(&["p", "span"])
        .add_tag_attributes("p", &["style"])
        .add_tag_attributes("span", &["style"])
        .filter_style_properties(
            HashSet::from(["font-size"])
        );

    let title = cleaner.clean(&title).to_string();
    let content = cleaner.clean(&content).to_string();

    let id = sqlx::query_scalar::<_, i64>("INSERT INTO notes (user_id, tab_id, order_id, title, content) VALUES (?, ?, ?, ?, ?) RETURNING id")
        .bind(session.user.id)
        .bind(tab_id)
        .bind(new_order_id)
        .bind(Vec::<u8>::new())
        .bind(Vec::<u8>::new())
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            error!("Failed to create note for user '{}': {:#?}", session.user.name, e);
            "Database error".to_string()
        })?;

    let dek = match session.get_dek() {
        Ok(dek) => dek,
        Err(e) => {
            state.session.clear_session(true)?;
            return Err(e);
        }
    };

    let enc_title = encrypt(&dek, title.as_bytes(), "notes.title", session.user.id, id)?;
    let enc_content = encrypt(&dek, content.as_bytes(), "notes.content", session.user.id, id)?;

    sqlx::query("UPDATE notes SET title = ?, content = ? WHERE id = ? AND user_id = ?")
        .bind(enc_title)
        .bind(enc_content)
        .bind(id)
        .bind(session.user.id)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            error!("Failed to update placeholder note data for user '{}': {:#?}", session.user.name, e);
            "Database error".to_string()
        })?;

    let note = Note {
        id,
        user_id: session.user.id,
        tab_id,
        order_id: new_order_id,
        title,
        content,
        bg_color: "#1c1c1c".into(),
    };

    tx.commit().await.map_err(|e| {
        error!("Failed to commit transaction: {:#?}", e);
        "Database error".to_string()
    })?;

    info!("User '{}' successfully added a note at {}", session.user.name, create_timestamp());

    let key = format!("{}-notes", session.user.id);

    match state.session.cache.contains(&key) {
        Ok(true) => {
            state.session.cache.update_cache(&key, &HashMap::from([(note.id, note.clone())]), &UpdateTask::Update, "Failed to add note to cache during note creation");
        },
        Ok(false) => {},
        Err(e) => {
            error!("CACHE POISONED ({}): Failed to check cache for user '{}'. Refetching data: {:#?}", create_timestamp(), session.user.name, e);
            fetch_and_cache_notes(state, &key, session.user.id, &session.user.name, &dek).await?;
        }
    }

    Ok(note)
}

#[tauri::command]
pub async fn get_notes(
    state: State<'_, AppState>,
) -> Result<Vec<Note>, String> {
    let state: &AppState = &state;

    let session: SessionData = state.session.get_session().map_err(|e| {
        error!("Fetching notes failed at {} due to: {:#?}", create_timestamp(), e);
        "An error occurred".to_string()
    })?;

    let dek = match session.get_dek() {
        Ok(dek) => dek,
        Err(e) => {
            state.session.clear_session(true)?;
            return Err(e);
        }
    };

    let key = format!("{}-notes", session.user.id);

    let mut notes = match state.session.cache.contains(&key) {
        Ok(true) => {
            match state.session.cache.get_notes(&key) {
                Ok(Some(notes)) => notes.values().cloned().collect(),
                Ok(None) => fetch_and_cache_notes(state, &key, session.user.id, &session.user.name, &dek).await?,
                Err(e) => {
                    error!("CACHE POISONED ({}): Failed to get notes from cache for user '{}': {:#?}", create_timestamp(), session.user.name, e);
                    fetch_and_cache_notes(state, &key, session.user.id, &session.user.name, &dek).await?
                }
            }
        },
        Ok(false) => fetch_and_cache_notes(state, &key, session.user.id, &session.user.name, &dek).await?,
        Err(e) => {
            error!("CACHE POISONED ({}): Failed to check cache for user '{}': {:#?}", create_timestamp(), session.user.name, e);
            fetch_and_cache_notes(state, &key, session.user.id, &session.user.name, &dek).await?
        }
    };

    notes.sort_by_key(|a| a.order_id);
    Ok(notes)
}

#[tauri::command]
pub async fn update_note(
    state: State<'_, AppState>,
    note_array: Vec<NoteUpdate>,
) -> Result<(), String> {
    let state: &AppState = &state;

    let session: SessionData = state.session.get_session().map_err(|e| {
        error!("Updating note failed at {} due to: {:#?}", create_timestamp(), e);
        "An error occurred".to_string()
    })?;

    if note_array.is_empty() {
        warn!("NOTE UPDATE FAILED ({}): User '{}' provided no notes", create_timestamp(), session.user.name);
        return Err("An error occurred".to_string());
    }

    let mut tx = state.db.begin().await.map_err(|e| {
        error!("Failed to begin transaction: {:#?}", e);
        "Database error".to_string()
    })?;

    let mut cleaner = ammonia::Builder::new();
    cleaner
        .add_tags(&["p", "span", "div", "h1", "h2", "h3", "label", "input", "ul", "li"])
        .add_tag_attributes("input", &["type"])
        .add_tag_attribute_values("input", "type", &["checkbox"])
        .add_tag_attributes("ul", &["data-type"])
        .add_tag_attribute_values("ul", "data-type", &["taskList"])
        .add_tag_attributes("li", &["data-checked", "data-type"])
        .add_tag_attribute_values("li", "data-type", &["taskItem"])
        .add_tag_attributes("p", &["style"])
        .add_tag_attributes("span", &["style"])
        .add_tag_attributes("div", &["style"])
        .add_tag_attributes("h1", &["style"])
        .add_tag_attributes("h2", &["style"])
        .add_tag_attributes("h3", &["style"])
        .filter_style_properties(
            HashSet::from(["text-align", "font-size", "background-color", "color"])
        );

    let dek = match session.get_dek() {
        Ok(dek) => dek,
        Err(e) => {
            state.session.clear_session(true)?;
            return Err(e);
        }
    };

    for note in &note_array {
        let mut qb: QueryBuilder<sqlx::Sqlite> = QueryBuilder::new("UPDATE notes SET ");
        let mut sep = qb.separated(", ");
        let mut has_changes = false;

        if let Some(title) = &note.title {
            sep.push("title = ").push_bind_unseparated(encrypt(&dek, cleaner.clean(title).to_string().as_bytes(), "notes.title", session.user.id, note.id)?);
            has_changes = true;
        }
        if let Some(content) = &note.content {
            sep.push("content = ").push_bind_unseparated(encrypt(&dek, &cleaner.clean(content).to_string().as_bytes(), "notes.content", session.user.id, note.id)?);
            has_changes = true;
        }
        if let Some(color) = &note.bg_color {
            sep.push("bg_color = ").push_bind_unseparated(cleaner.clean(color).to_string());
            has_changes = true;
        }

        if !has_changes {
            continue;
        }

        qb.push(" WHERE id = ").push_bind(note.id);
        qb.push(" AND user_id = ").push_bind(session.user.id);

        qb.build()
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                error!("Failed to update note '{}' for user '{}': {:#?}", note.id, session.user.name, e);
                "Database error".to_string()
            })?;
    }

    tx.commit().await.map_err(|e| {
        error!("Failed to commit transaction: {:#?}", e);
        "Database error".to_string()
    })?;

    let placeholders: Vec<&str> = (0..note_array.len()).map(|_| "?").collect();
    let select_query = format!("SELECT * FROM notes WHERE user_id = ? AND id IN ({})", placeholders.join(", "));
    let mut select_query = query_as::<_, StoredNote>(&select_query).bind(session.user.id);

    for note in &note_array {
        select_query = select_query.bind(note.id);
    }

    let stored = select_query.fetch_all(&state.db).await.map_err(|e| {
        error!("Failed to fetch updated notes for user '{}': {:#?}", session.user.name, e);
        "Database error".to_string()
    })?;

    let mut updated_notes = HashMap::<i64, Note>::new();
    for note in stored {
        let title = String::from_utf8(decrypt(&dek, &note.title, "notes.title", session.user.id, note.id)?).map_err(|e| {
            error!("Failed to parse note title during update: {:#?}", e);
            "An error occurred".to_string()
        })?;
        let content = String::from_utf8(decrypt(&dek, &note.content, "notes.content", session.user.id, note.id)?).map_err(|e| {
            error!("Failed to parse note content during update: {:#?}", e);
            "An error occurred".to_string()
        })?;

        updated_notes.insert(
            note.id,
            Note {
                id: note.id,
                user_id: note.user_id,
                tab_id: note.tab_id,
                order_id: note.order_id,
                title,
                content,
                bg_color: note.bg_color
            }
        );
    }

    let key = format!("{}-notes", session.user.id);
    state.session.cache.update_cache(&key, &updated_notes, &UpdateTask::Update, "Failed to update note in cache during note update");

    Ok(())
}

#[tauri::command]
pub async fn delete_note(
    state: State<'_, AppState>,
    note_id: i64,
) -> Result<(), String> {
    let state: &AppState = &state;

    let session: SessionData = state.session.get_session().map_err(|e| {
        error!("Deleting note failed at {} due to: {:#?}", create_timestamp(), e);
        "An error occurred".to_string()
    })?;

    let stored = query_as::<_, StoredNote>("DELETE FROM notes WHERE id = ? AND user_id = ? RETURNING *")
        .bind(note_id)
        .bind(session.user.id)
        .fetch_one(&state.db)
        .await
        .map_err(|e|{
            error!("Failed to delete note {} from user '{}': {:#?}", note_id, session.user.name, e);
            "Database error".to_string()
        })?;

    let dek = match session.get_dek() {
        Ok(dek) => dek,
        Err(e) => {
            state.session.clear_session(true)?;
            return Err(e);
        }
    };

    let title = String::from_utf8(decrypt(&dek, &stored.title, "notes.title", session.user.id, stored.id)?).map_err(|e| {
        error!("Failed to parse note title during deletion: {:#?}", e);
        "An error occurred".to_string()
    })?;
    let content = String::from_utf8(decrypt(&dek, &stored.content, "notes.content", session.user.id, stored.id)?).map_err(|e| {
        error!("Failed to parse note content during deletion: {:#?}", e);
        "An error occurred".to_string()
    })?;

    let note = Note {
        id: stored.id,
        user_id: stored.user_id,
        tab_id: stored.tab_id,
        order_id: stored.order_id,
        title,
        content,
        bg_color: stored.bg_color,
    };

    info!("User '{}' successfully deleted a note at {}", session.user.name, create_timestamp());

    let key = format!("{}-notes", session.user.id);

    state.session.cache.update_cache(&key, &HashMap::from([(note.id, note)]), &UpdateTask::Delete, "Failed to delete note from cache on note deletion");

    Ok(())
}

#[tauri::command]
pub async fn create_tab(
    state: State<'_, AppState>,
    title: String,
) -> Result<Tab, String> {
    let state: &AppState = &state;

    let session: SessionData = state.session.get_session().map_err(|e| {
        error!("Creating tab failed at {} due to: {:#?}", create_timestamp(), e);
        "An error occurred".to_string()
    })?;

    let mut tx = state.db.begin().await.map_err(|e| {
        error!("Failed to start transaction: {:#?}", e);
        "Database error".to_string()
    })?;

    let max_order_id = sqlx::query_scalar::<_, u32>("SELECT MAX(order_id) FROM tabs WHERE user_id = ?")
        .bind(session.user.id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| {
            error!("Failed to fetch the current highest order ID for creating tab: {:#?}", e);
            "Database error".to_string()
        })?
        .unwrap_or(0);

    let new_order_id = max_order_id + 1;

    let title = ammonia::clean(&title);

    let tab = query_as::<_, Tab>("INSERT INTO tabs (user_id, order_id, title) VALUES (?, ?, ?) RETURNING *")
        .bind(session.user.id)
        .bind(new_order_id)
        .bind(title)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            error!("Failed to create tab for user '{}': {:#?}", session.user.name, e);
            "Database error".to_string()
        })?;

    tx.commit().await.map_err(|e| {
        error!("Failed to commit transaction: {:#?}", e);
        "Database error".to_string()
    })?;

    info!("User '{}' successfully added a new tab at {}", session.user.name, create_timestamp());

    Ok(tab)
}

#[tauri::command]
pub async fn get_tabs(
    state: State<'_, AppState>,
) -> Result<Vec<Tab>, String> {
    let state: &AppState = &state;

    let session: SessionData = state.session.get_session().map_err(|e| {
        error!("Fetching tabs failed at {} due to: {:#?}", create_timestamp(), e);
        "An error occurred".to_string()
    })?;

    let tabs = query_as::<_, Tab>("SELECT * FROM tabs WHERE user_id = ? ORDER BY order_id ASC")
        .bind(session.user.id)
        .fetch_all(&state.db)
        .await
        .map_err(|e| {
            error!("Failed to fetch tabs for user '{}': {:#?}", session.user.name, e);
            "Database error".to_string()
        })?;

    Ok(tabs)
}

#[tauri::command]
pub async fn update_tab(
    state: State<'_, AppState>,
    tab_id: i64,
    title: String,
) -> Result<TabIdTitle, String> {
    let state: &AppState = &state;

    let session: SessionData = state.session.get_session().map_err(|e| {
        error!("Updating tab failed at {} due to: {:#?}", create_timestamp(), e);
        "An error occurred".to_string()
    })?;

    if tab_id.le(&0) {
        error!("TAB UPDATE FAILED ({}): User '{}' tried updating a tab with an invalid tab ID: '{}'", create_timestamp(), session.user.name, tab_id);
        return Err("An error occurred".to_string());
    }

    if title.trim().is_empty() {
        error!("TAB UPDATE FAILED ({}): User '{}' did not give a name for a tab", create_timestamp(), session.user.name);
        return Err("No name for tab".to_string());
    }

    let title = ammonia::clean(&title);

    let tab = query_as::<_, TabIdTitle>("UPDATE tabs SET title = ? WHERE user_id = ? AND id = ? RETURNING id, title")
        .bind(title)
        .bind(session.user.id)
        .bind(tab_id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| {
            error!("Failed to update tab with ID {}, by user '{}': {:#?}", tab_id, session.user.name, e);
            "Database error".to_string()
        })?;

    Ok(tab)
}

#[tauri::command]
pub async fn update_tab_color(
    state: State<'_, AppState>,
    tab_id: i64,
    color: String,
) -> Result<Tab, String> {
    let state: &AppState = &state;

    let session: SessionData = state.session.get_session().map_err(|e| {
        error!("Updating tab color failed at {} due to: {:#?}", create_timestamp(), e);
        "An error occurred".to_string()
    })?;

    if tab_id.le(&0) {
        error!("TAB UPDATE FAILED ({}): User '{}' tried updating a tab with an invalid tab ID: '{}'", create_timestamp(), session.user.name, tab_id);
        return Err("An error occurred".to_string());
    }

    if color.trim().is_empty() {
        error!("User '{}' provided no color for tab", session.user.name);
        return Err("No color provided".to_string());
    }

    let color = ammonia::clean(&color);

    let tab = query_as::<_, Tab>("UPDATE tabs SET color = ? WHERE user_id = ? AND id = ? RETURNING *")
        .bind(color)
        .bind(session.user.id)
        .bind(tab_id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| {
            error!("Failed to update tab color for user '{}': {:#?}", session.user.name, e);
            "Database error".to_string()
        })?;

    Ok(tab)
}

#[tauri::command]
pub async fn delete_tab(
    state: State<'_, AppState>,
    tab_id: i64,
) -> Result<Tab, String> {
    let state: &AppState = &state;

    let session: SessionData = state.session.get_session().map_err(|e| {
        error!("Deleting tab failed at {} due to: {:#?}", create_timestamp(), e);
        "An error occurred".to_string()
    })?;

    if tab_id.le(&0) {
        error!("TAB DELETION FAILED ({}): User '{}' tried deleting a tab with an invalid tab ID: '{}'", create_timestamp(), session.user.name, tab_id);
        return Err("An error occurred".to_string());
    }

    let tab = query_as::<_, Tab>("DELETE FROM tabs WHERE user_id = ? AND id = ? RETURNING *")
        .bind(session.user.id)
        .bind(tab_id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| {
            error!("Failed to delete tab with ID '{}' for user '{}': {:#?}", tab_id, session.user.name, e);
            "Database error".to_string()
        })?;

    info!("User '{}' successfully deleted a tab at {}", session.user.name, create_timestamp());

    Ok(tab)
}