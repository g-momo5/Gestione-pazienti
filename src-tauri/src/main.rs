// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod database;
mod models;

use database::Database;
use std::path::PathBuf;

fn main() {
    // Determina il percorso del database
    let db_path = get_database_path();

    // Inizializza il database
    let db = Database::new(db_path).expect("Failed to initialize database");

    tauri::Builder::default()
        .manage(db)
        .invoke_handler(tauri::generate_handler![
            commands::get_all_procedures,
            commands::get_procedure_by_id,
            commands::create_procedure,
            commands::update_procedure,
            commands::delete_procedure,
            commands::calculate_statistics,
            commands::get_procedure_count,
            commands::get_all_patients,
            commands::get_patient_by_id,
            commands::create_patient,
            commands::update_patient,
            commands::delete_patient,
            commands::change_patient_status,
            commands::get_patient_status_counts,
            commands::get_patients_by_status,
            commands::generate_ambulatorio_referto,
            commands::generate_scheda_procedurale_referto,
            commands::load_settings,
            commands::save_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Determina il percorso del database nell'app data directory
fn get_database_path() -> PathBuf {
    let app_data_dir = tauri::api::path::app_data_dir(&tauri::Config::default())
        .unwrap_or_else(|| {
            // Fallback a directory locale se non disponibile
            PathBuf::from(".")
        });

    // Crea la directory se non esiste
    std::fs::create_dir_all(&app_data_dir).ok();

    // Rimuovi il vecchio database se esiste
    let old_db = app_data_dir.join("registro_tavi.db");
    if old_db.exists() {
        let _ = std::fs::remove_file(&old_db);
    }

    // Usa il nuovo database per il gestionale pazienti
    app_data_dir.join("pazienti_tavi.db")
}
