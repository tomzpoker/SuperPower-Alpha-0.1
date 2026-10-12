mod commands;
mod database;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_data_dir)?;
            app.manage(database::AppState::new(app_data_dir.join("superpower.db")));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::database_status,
            commands::initialize_database,
            commands::unlock_database,
            commands::lock_database,
            commands::list_societes,
            commands::create_societe,
            commands::update_societe,
            commands::delete_societe,
            commands::list_flux,
            commands::create_flux,
            commands::update_flux,
            commands::cancel_flux,
            commands::register_flux_realisation,
            commands::cancel_flux_realisation,
            commands::list_comptes,
            commands::create_ecriture,
            commands::list_ecritures,
            commands::dashboard_summary,
        ])
        .run(tauri::generate_context!())
        .expect("Erreur au démarrage de SuperPower Alpha");
}
