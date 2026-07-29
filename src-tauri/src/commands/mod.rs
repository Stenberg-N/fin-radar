use tauri::generate_handler;
use tauri::ipc::Invoke;

pub mod helpers;
pub mod user;
pub mod transactions;
pub mod others;
pub mod notes;
pub mod timers;
pub mod calendar;

pub fn all_handlers () -> impl Fn(Invoke) -> bool + Send + Sync + 'static {
    generate_handler![
        user::create_user,
        user::login_user,
        user::delete_user,
        user::change_password,
        user::recover_password,
        user::cancel_password_recovery,
        user::logout_user,
        user::update_user_session,
        others::backup_database,
        others::reorder_array,
        transactions::add_transaction,
        transactions::get_transactions,
        transactions::delete_transaction,
        transactions::update_transaction,
        transactions::get_year_transactions,
        notes::create_note,
        notes::get_notes,
        notes::update_note,
        notes::delete_note,
        notes::create_tab,
        notes::get_tabs,
        notes::update_tab,
        notes::delete_tab,
        notes::update_tab_color,
        timers::create_timer,
        timers::get_timers,
        timers::update_timer,
        timers::delete_timer,
        calendar::add_calendar_event,
        calendar::get_calendar_events,
    ]
}