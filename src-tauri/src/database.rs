use rusqlite::{Connection, params, Result as SqlResult};
use std::path::PathBuf;
use std::sync::Mutex;
use crate::models::{Procedure, ProcedureFilters, Statistics, Patient, PatientStatus, PatientWithStatus, PatientFilters, PatientStatusCount};
use serde_json;

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    /// Crea una nuova connessione al database
    pub fn new(db_path: PathBuf) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(db_path)?;
        let db = Database {
            conn: Mutex::new(conn),
        };
        db.initialize_schema()?;
        Ok(db)
    }

    /// Crea le tabelle di stato se mancanti (migrazione difensiva).
    fn ensure_status_tables(&self, conn: &Connection) -> SqlResult<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS patients_da_valutare (
                patient_id INTEGER PRIMARY KEY,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY(patient_id) REFERENCES patients(id) ON DELETE CASCADE
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS patients_in_attesa_esami (
                patient_id INTEGER PRIMARY KEY,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY(patient_id) REFERENCES patients(id) ON DELETE CASCADE
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS patients_in_attesa_intervento (
                patient_id INTEGER PRIMARY KEY,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY(patient_id) REFERENCES patients(id) ON DELETE CASCADE
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS patients_non_candidabile (
                patient_id INTEGER PRIMARY KEY,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY(patient_id) REFERENCES patients(id) ON DELETE CASCADE
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS patients_completato (
                patient_id INTEGER PRIMARY KEY,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY(patient_id) REFERENCES patients(id) ON DELETE CASCADE
            )",
            [],
        )?;

        Ok(())
    }

    /// Inizializza lo schema del database
    fn initialize_schema(&self) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();

        // Tabella pazienti (anagrafica)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS patients (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
                nome TEXT NOT NULL,
                cognome TEXT NOT NULL,
                data_nascita TEXT NOT NULL,
                luogo_nascita TEXT,
                codice_fiscale TEXT,
                telefono TEXT,
                email TEXT,
                provenienza TEXT,
                sesso TEXT,
                altezza REAL,
                peso REAL,
                note TEXT,
                ambulatorio_fattori TEXT,
                anamnesi_cardiologica TEXT,
                apr TEXT,
                visita_odierna TEXT,
                conclusioni TEXT,
                medico_titolo TEXT,
                medico_nome TEXT,
                medico_specializzando_titolo TEXT,
                medico_specializzando_nome TEXT,
                procedurale_allergia_mdc TEXT,
                procedurale_preparazione_mdc TEXT,
                procedurale_creatinina TEXT,
                procedurale_egfr TEXT,
                procedurale_hb TEXT,
                procedurale_altro TEXT,
                data_tavi TEXT,
                procedurale_ecg_ritmo_sinusale INTEGER,
                procedurale_ecg_fa INTEGER,
                procedurale_ecg_bbs INTEGER,
                procedurale_ecg_bbd INTEGER,
                procedurale_ecg_eas INTEGER,
                procedurale_ecg_bav_primo INTEGER,
                procedurale_ecg_ritmo_stimolato INTEGER,
                procedurale_anestesia TEXT,
                procedurale_coronarografia TEXT,
                procedurale_coronarografia_note TEXT,
                procedurale_pacemaker TEXT,
                procedurale_pacemaker_note TEXT,
                procedurale_accesso_principale_fem TEXT,
                procedurale_accesso_principale_altro TEXT,
                procedurale_accesso_protezione TEXT,
                procedurale_accesso_protezione_note TEXT,
                procedurale_altri_accessi TEXT,
                procedurale_diametro_pallone_femorale TEXT,
                procedurale_guida_safari TEXT,
                procedurale_protezione_osti TEXT,
                procedurale_valvuloplastica TEXT,
                procedurale_valvuloplastica_note TEXT,
                procedurale_bioprotesi_modello TEXT,
                procedurale_bioprotesi_dimensione TEXT
            )",
            [],
        )?;
        // Migrazione soft per db esistenti: ignora errore se la colonna esiste già
        let _ = conn.execute("ALTER TABLE patients ADD COLUMN sesso TEXT", []);
        let _ = conn.execute("ALTER TABLE patients ADD COLUMN altezza REAL", []);
        let _ = conn.execute("ALTER TABLE patients ADD COLUMN peso REAL", []);
        let _ = conn.execute("ALTER TABLE patients ADD COLUMN note TEXT", []);
        let _ = conn.execute("ALTER TABLE patients ADD COLUMN ambulatorio_fattori TEXT", []);
        let _ = conn.execute("ALTER TABLE patients ADD COLUMN anamnesi_cardiologica TEXT", []);
        let _ = conn.execute("ALTER TABLE patients ADD COLUMN apr TEXT", []);
        let _ = conn.execute("ALTER TABLE patients ADD COLUMN visita_odierna TEXT", []);
        let _ = conn.execute("ALTER TABLE patients ADD COLUMN conclusioni TEXT", []);
        let _ = conn.execute("ALTER TABLE patients ADD COLUMN medico_titolo TEXT", []);
        let _ = conn.execute("ALTER TABLE patients ADD COLUMN medico_nome TEXT", []);
        let _ = conn.execute("ALTER TABLE patients ADD COLUMN medico_specializzando_titolo TEXT", []);
        let _ = conn.execute("ALTER TABLE patients ADD COLUMN medico_specializzando_nome TEXT", []);
        let _ = conn.execute("ALTER TABLE patients ADD COLUMN procedurale_allergia_mdc TEXT", []);
        let _ = conn.execute("ALTER TABLE patients ADD COLUMN procedurale_preparazione_mdc TEXT", []);
        let _ = conn.execute("ALTER TABLE patients ADD COLUMN procedurale_creatinina TEXT", []);
        let _ = conn.execute("ALTER TABLE patients ADD COLUMN procedurale_egfr TEXT", []);
        let _ = conn.execute("ALTER TABLE patients ADD COLUMN procedurale_hb TEXT", []);
        let _ = conn.execute("ALTER TABLE patients ADD COLUMN procedurale_altro TEXT", []);
        let _ = conn.execute("ALTER TABLE patients ADD COLUMN data_tavi TEXT", []);
        let _ = conn.execute("ALTER TABLE patients ADD COLUMN procedurale_ecg_ritmo_sinusale INTEGER", []);
        let _ = conn.execute("ALTER TABLE patients ADD COLUMN procedurale_ecg_fa INTEGER", []);
        let _ = conn.execute("ALTER TABLE patients ADD COLUMN procedurale_ecg_bbs INTEGER", []);
        let _ = conn.execute("ALTER TABLE patients ADD COLUMN procedurale_ecg_bbd INTEGER", []);
        let _ = conn.execute("ALTER TABLE patients ADD COLUMN procedurale_ecg_eas INTEGER", []);
        let _ = conn.execute("ALTER TABLE patients ADD COLUMN procedurale_ecg_bav_primo INTEGER", []);
        let _ = conn.execute("ALTER TABLE patients ADD COLUMN procedurale_ecg_ritmo_stimolato INTEGER", []);
        let _ = conn.execute("ALTER TABLE patients ADD COLUMN procedurale_anestesia TEXT", []);
        let _ = conn.execute("ALTER TABLE patients ADD COLUMN procedurale_coronarografia TEXT", []);
        let _ = conn.execute("ALTER TABLE patients ADD COLUMN procedurale_coronarografia_note TEXT", []);
        let _ = conn.execute("ALTER TABLE patients ADD COLUMN procedurale_pacemaker TEXT", []);
        let _ = conn.execute("ALTER TABLE patients ADD COLUMN procedurale_pacemaker_note TEXT", []);
        let _ = conn.execute("ALTER TABLE patients ADD COLUMN procedurale_accesso_principale_fem TEXT", []);
        let _ = conn.execute("ALTER TABLE patients ADD COLUMN procedurale_accesso_principale_altro TEXT", []);
        let _ = conn.execute("ALTER TABLE patients ADD COLUMN procedurale_accesso_protezione TEXT", []);
        let _ = conn.execute("ALTER TABLE patients ADD COLUMN procedurale_accesso_protezione_note TEXT", []);
        let _ = conn.execute("ALTER TABLE patients ADD COLUMN procedurale_altri_accessi TEXT", []);
        let _ = conn.execute("ALTER TABLE patients ADD COLUMN procedurale_diametro_pallone_femorale TEXT", []);
        let _ = conn.execute("ALTER TABLE patients ADD COLUMN procedurale_guida_safari TEXT", []);
        let _ = conn.execute("ALTER TABLE patients ADD COLUMN procedurale_protezione_osti TEXT", []);
        let _ = conn.execute("ALTER TABLE patients ADD COLUMN procedurale_valvuloplastica TEXT", []);
        let _ = conn.execute("ALTER TABLE patients ADD COLUMN procedurale_valvuloplastica_note TEXT", []);
        let _ = conn.execute("ALTER TABLE patients ADD COLUMN procedurale_bioprotesi_modello TEXT", []);
        let _ = conn.execute("ALTER TABLE patients ADD COLUMN procedurale_bioprotesi_dimensione TEXT", []);

        // Tabelle per stato di avanzamento (una per stato)
        self.ensure_status_tables(&conn)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS procedures (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP,

                -- DATI PAZIENTE
                nome TEXT NOT NULL,
                cognome TEXT NOT NULL,
                data_nascita TEXT NOT NULL,
                altezza REAL,
                peso REAL,

                -- DATI PRE-PROCEDURALI
                fe REAL,
                vmax REAL,
                gmax REAL,
                gmed REAL,
                ava REAL,
                anulus_aortico REAL,
                valvola_protesica INTEGER DEFAULT 0,
                protesica_modello TEXT,
                protesica_dimensione TEXT,

                -- DATI PROCEDURALI
                data_procedura TEXT NOT NULL,
                ora_inizio TEXT NOT NULL,
                ora_fine TEXT NOT NULL,
                tipo_valvola TEXT NOT NULL CHECK(tipo_valvola IN ('Balloon Expandable', 'Self Expandable')),
                modello_valvola TEXT NOT NULL,
                dimensione_valvola REAL,
                pre_dilatazione INTEGER DEFAULT 0,
                post_dilatazione INTEGER DEFAULT 0
            )",
            [],
        )?;

        // Crea indici per performance
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_nome_cognome ON procedures(nome, cognome)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_data_procedura ON procedures(data_procedura)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_tipo_valvola ON procedures(tipo_valvola)",
            [],
        )?;

        Ok(())
    }

    /// Inserisce una nuova procedura
    pub fn insert_procedure(&self, proc: &Procedure) -> Result<i64, String> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "INSERT INTO procedures (
                nome, cognome, data_nascita, altezza, peso,
                fe, vmax, gmax, gmed, ava, anulus_aortico,
                valvola_protesica, protesica_modello, protesica_dimensione,
                data_procedura, ora_inizio, ora_fine,
                tipo_valvola, modello_valvola, dimensione_valvola,
                pre_dilatazione, post_dilatazione
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22)",
            params![
                proc.nome, proc.cognome, proc.data_nascita, proc.altezza, proc.peso,
                proc.fe, proc.vmax, proc.gmax, proc.gmed, proc.ava, proc.anulus_aortico,
                proc.valvola_protesica, proc.protesica_modello, proc.protesica_dimensione,
                proc.data_procedura, proc.ora_inizio, proc.ora_fine,
                proc.tipo_valvola, proc.modello_valvola, proc.dimensione_valvola,
                proc.pre_dilatazione, proc.post_dilatazione
            ],
        ).map_err(|e| e.to_string())?;

        Ok(conn.last_insert_rowid())
    }

    /// Aggiorna una procedura esistente
    pub fn update_procedure(&self, proc: &Procedure) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();

        let id = proc.id.ok_or("Procedure ID is required for update")?;

        conn.execute(
            "UPDATE procedures SET
                nome = ?1, cognome = ?2, data_nascita = ?3, altezza = ?4, peso = ?5,
                fe = ?6, vmax = ?7, gmax = ?8, gmed = ?9, ava = ?10, anulus_aortico = ?11,
                valvola_protesica = ?12, protesica_modello = ?13, protesica_dimensione = ?14,
                data_procedura = ?15, ora_inizio = ?16, ora_fine = ?17,
                tipo_valvola = ?18, modello_valvola = ?19, dimensione_valvola = ?20,
                pre_dilatazione = ?21, post_dilatazione = ?22,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = ?23",
            params![
                proc.nome, proc.cognome, proc.data_nascita, proc.altezza, proc.peso,
                proc.fe, proc.vmax, proc.gmax, proc.gmed, proc.ava, proc.anulus_aortico,
                proc.valvola_protesica, proc.protesica_modello, proc.protesica_dimensione,
                proc.data_procedura, proc.ora_inizio, proc.ora_fine,
                proc.tipo_valvola, proc.modello_valvola, proc.dimensione_valvola,
                proc.pre_dilatazione, proc.post_dilatazione,
                id
            ],
        ).map_err(|e| e.to_string())?;

        Ok(())
    }

    /// Elimina una procedura
    pub fn delete_procedure(&self, id: i64) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();

        conn.execute("DELETE FROM procedures WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    /// Ottieni tutte le procedure (con filtri opzionali)
    pub fn get_all_procedures(&self, filters: Option<ProcedureFilters>) -> Result<Vec<Procedure>, String> {
        let conn = self.conn.lock().unwrap();

        let mut query = String::from("SELECT * FROM procedures WHERE 1=1");
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![];

        if let Some(filters) = filters {
            // Filtro ricerca testuale
            if let Some(search) = filters.search_query {
                if !search.is_empty() {
                    query.push_str(" AND (nome LIKE ?1 OR cognome LIKE ?1 OR modello_valvola LIKE ?1)");
                    params.push(Box::new(format!("%{}%", search)));
                }
            }

            // Filtro tipo valvola
            if let Some(tipo) = filters.tipo_valvola {
                if tipo != "all" {
                    query.push_str(&format!(" AND tipo_valvola = ?{}", params.len() + 1));
                    params.push(Box::new(tipo));
                }
            }

            // Filtro periodo
            if let Some(period) = filters.period {
                if period != "all" {
                    let days = match period.as_str() {
                        "1m" => 30,
                        "3m" => 90,
                        "6m" => 180,
                        "1y" => 365,
                        _ => 0,
                    };

                    if days > 0 {
                        query.push_str(&format!(" AND data_procedura >= date('now', '-{} days')", days));
                    }
                }
            }
        }

        query.push_str(" ORDER BY data_procedura DESC, ora_inizio DESC");

        let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;
        let rows = stmt.query_map(params_refs.as_slice(), |row| {
            Ok(Procedure {
                id: Some(row.get(0)?),
                created_at: row.get(1).ok(),
                updated_at: row.get(2).ok(),
                nome: row.get(3)?,
                cognome: row.get(4)?,
                data_nascita: row.get(5)?,
                altezza: row.get(6).ok(),
                peso: row.get(7).ok(),
                fe: row.get(8).ok(),
                vmax: row.get(9).ok(),
                gmax: row.get(10).ok(),
                gmed: row.get(11).ok(),
                ava: row.get(12).ok(),
                anulus_aortico: row.get(13).ok(),
                valvola_protesica: row.get::<_, i32>(14)? != 0,
                protesica_modello: row.get(15).ok(),
                protesica_dimensione: row.get(16).ok(),
                data_procedura: row.get(17)?,
                ora_inizio: row.get(18)?,
                ora_fine: row.get(19)?,
                tipo_valvola: row.get(20)?,
                modello_valvola: row.get(21)?,
                dimensione_valvola: row.get(22).ok(),
                pre_dilatazione: row.get::<_, i32>(23)? != 0,
                post_dilatazione: row.get::<_, i32>(24)? != 0,
            })
        }).map_err(|e| e.to_string())?;

        let procedures: Result<Vec<_>, _> = rows.collect();
        procedures.map_err(|e| e.to_string())
    }

    /// Ottieni una procedura per ID
    pub fn get_procedure_by_id(&self, id: i64) -> Result<Option<Procedure>, String> {
        let conn = self.conn.lock().unwrap();

        let result = conn.query_row(
            "SELECT * FROM procedures WHERE id = ?1",
            params![id],
            |row| {
                Ok(Procedure {
                    id: Some(row.get(0)?),
                    created_at: row.get(1).ok(),
                    updated_at: row.get(2).ok(),
                    nome: row.get(3)?,
                    cognome: row.get(4)?,
                    data_nascita: row.get(5)?,
                    altezza: row.get(6).ok(),
                    peso: row.get(7).ok(),
                    fe: row.get(8).ok(),
                    vmax: row.get(9).ok(),
                    gmax: row.get(10).ok(),
                    gmed: row.get(11).ok(),
                    ava: row.get(12).ok(),
                    anulus_aortico: row.get(13).ok(),
                    valvola_protesica: row.get::<_, i32>(14)? != 0,
                    protesica_modello: row.get(15).ok(),
                    protesica_dimensione: row.get(16).ok(),
                    data_procedura: row.get(17)?,
                    ora_inizio: row.get(18)?,
                    ora_fine: row.get(19)?,
                    tipo_valvola: row.get(20)?,
                    modello_valvola: row.get(21)?,
                    dimensione_valvola: row.get(22).ok(),
                    pre_dilatazione: row.get::<_, i32>(23)? != 0,
                    post_dilatazione: row.get::<_, i32>(24)? != 0,
                })
            },
        );

        match result {
            Ok(proc) => Ok(Some(proc)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.to_string()),
        }
    }

    /// Calcola le statistiche
    pub fn calculate_statistics(&self, filters: Option<ProcedureFilters>) -> Result<Statistics, String> {
        let procedures = self.get_all_procedures(filters)?;

        let total = procedures.len() as i32;

        if total == 0 {
            return Ok(Statistics {
                total_procedures: 0,
                average_duration_minutes: 0.0,
                pre_dilatazione_percentage: 0.0,
                post_dilatazione_percentage: 0.0,
                average_fe: None,
                average_vmax: None,
                average_gmax: None,
                average_gmed: None,
                average_ava: None,
                balloon_expandable_count: 0,
                self_expandable_count: 0,
                top_valve_models: vec![],
            });
        }

        // Calcola medie
        let mut total_duration = 0;
        let mut pre_count = 0;
        let mut post_count = 0;
        let mut balloon_count = 0;
        let mut self_count = 0;

        let mut fe_sum = 0.0;
        let mut fe_count = 0;
        let mut vmax_sum = 0.0;
        let mut vmax_count = 0;
        let mut gmax_sum = 0.0;
        let mut gmax_count = 0;
        let mut gmed_sum = 0.0;
        let mut gmed_count = 0;
        let mut ava_sum = 0.0;
        let mut ava_count = 0;

        let mut model_counts: std::collections::HashMap<String, i32> = std::collections::HashMap::new();

        for proc in &procedures {
            if let Some(duration) = proc.calculate_duration_minutes() {
                total_duration += duration;
            }

            if proc.pre_dilatazione {
                pre_count += 1;
            }

            if proc.post_dilatazione {
                post_count += 1;
            }

            if proc.tipo_valvola == "Balloon Expandable" {
                balloon_count += 1;
            } else {
                self_count += 1;
            }

            if let Some(fe) = proc.fe {
                fe_sum += fe;
                fe_count += 1;
            }
            if let Some(vmax) = proc.vmax {
                vmax_sum += vmax;
                vmax_count += 1;
            }
            if let Some(gmax) = proc.gmax {
                gmax_sum += gmax;
                gmax_count += 1;
            }
            if let Some(gmed) = proc.gmed {
                gmed_sum += gmed;
                gmed_count += 1;
            }
            if let Some(ava) = proc.ava {
                ava_sum += ava;
                ava_count += 1;
            }

            *model_counts.entry(proc.modello_valvola.clone()).or_insert(0) += 1;
        }

        // Top 5 modelli
        let mut top_models: Vec<(String, i32)> = model_counts.into_iter().collect();
        top_models.sort_by(|a, b| b.1.cmp(&a.1));
        top_models.truncate(5);

        Ok(Statistics {
            total_procedures: total,
            average_duration_minutes: if total > 0 {
                total_duration as f64 / total as f64
            } else {
                0.0
            },
            pre_dilatazione_percentage: (pre_count as f64 / total as f64) * 100.0,
            post_dilatazione_percentage: (post_count as f64 / total as f64) * 100.0,
            average_fe: if fe_count > 0 {
                Some(fe_sum / fe_count as f64)
            } else {
                None
            },
            average_vmax: if vmax_count > 0 {
                Some(vmax_sum / vmax_count as f64)
            } else {
                None
            },
            average_gmax: if gmax_count > 0 {
                Some(gmax_sum / gmax_count as f64)
            } else {
                None
            },
            average_gmed: if gmed_count > 0 {
                Some(gmed_sum / gmed_count as f64)
            } else {
                None
            },
            average_ava: if ava_count > 0 {
                Some(ava_sum / ava_count as f64)
            } else {
                None
            },
            balloon_expandable_count: balloon_count,
            self_expandable_count: self_count,
            top_valve_models: top_models,
        })
    }

    // ========================================================================
    // PATIENT OPERATIONS
    // ========================================================================

    /// Inserisce un nuovo paziente (stato iniziale: Da valutare)
    pub fn insert_patient(&self, patient: &Patient) -> Result<i64, String> {
        let conn = self.conn.lock().unwrap();

        conn.execute("BEGIN TRANSACTION", []).map_err(|e| e.to_string())?;

        let fattori_json = patient.ambulatorio_fattori.as_ref().and_then(|f| serde_json::to_string(f).ok());

        let result = conn.execute(
            "INSERT INTO patients (
                nome, cognome, data_nascita, luogo_nascita, codice_fiscale, telefono, email, provenienza, sesso, altezza, peso, note,
                ambulatorio_fattori, anamnesi_cardiologica, apr, visita_odierna, conclusioni, medico_titolo, medico_nome, medico_specializzando_titolo, medico_specializzando_nome,
                procedurale_allergia_mdc, procedurale_preparazione_mdc, procedurale_creatinina, procedurale_egfr, procedurale_hb, procedurale_altro, data_tavi,
                procedurale_ecg_ritmo_sinusale, procedurale_ecg_fa, procedurale_ecg_bbs, procedurale_ecg_bbd, procedurale_ecg_eas, procedurale_ecg_bav_primo, procedurale_ecg_ritmo_stimolato,
                procedurale_anestesia, procedurale_coronarografia, procedurale_coronarografia_note, procedurale_pacemaker, procedurale_pacemaker_note,
                procedurale_accesso_principale_fem, procedurale_accesso_principale_altro, procedurale_accesso_protezione, procedurale_accesso_protezione_note,
                procedurale_altri_accessi, procedurale_diametro_pallone_femorale, procedurale_guida_safari, procedurale_protezione_osti,
                procedurale_valvuloplastica, procedurale_valvuloplastica_note, procedurale_bioprotesi_modello, procedurale_bioprotesi_dimensione
            )
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20,
                     ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28, ?29, ?30, ?31, ?32, ?33,
                     ?34, ?35, ?36, ?37, ?38, ?39, ?40, ?41, ?42, ?43, ?44, ?45, ?46, ?47, ?48, ?49, ?50, ?51, ?52)",
            params![
                patient.nome,
                patient.cognome,
                patient.data_nascita,
                patient.luogo_nascita,
                patient.codice_fiscale,
                patient.telefono,
                patient.email,
                patient.provenienza,
                patient.sesso,
                patient.altezza,
                patient.peso,
                patient.note,
                fattori_json,
                patient.anamnesi_cardiologica,
                patient.apr,
                patient.visita_odierna,
                patient.conclusioni,
                patient.medico_titolo,
                patient.medico_nome,
                patient.medico_specializzando_titolo,
                patient.medico_specializzando_nome,
                patient.procedurale_allergia_mdc,
                patient.procedurale_preparazione_mdc,
                patient.procedurale_creatinina,
                patient.procedurale_egfr,
                patient.procedurale_hb,
                patient.procedurale_altro,
                patient.data_tavi,
                patient.procedurale_ecg_ritmo_sinusale,
                patient.procedurale_ecg_fa,
                patient.procedurale_ecg_bbs,
                patient.procedurale_ecg_bbd,
                patient.procedurale_ecg_eas,
                patient.procedurale_ecg_bav_primo,
                patient.procedurale_ecg_ritmo_stimolato,
                patient.procedurale_anestesia,
                patient.procedurale_coronarografia,
                patient.procedurale_coronarografia_note,
                patient.procedurale_pacemaker,
                patient.procedurale_pacemaker_note,
                patient.procedurale_accesso_principale_fem,
                patient.procedurale_accesso_principale_altro,
                patient.procedurale_accesso_protezione,
                patient.procedurale_accesso_protezione_note,
                patient.procedurale_altri_accessi,
                patient.procedurale_diametro_pallone_femorale,
                patient.procedurale_guida_safari,
                patient.procedurale_protezione_osti,
                patient.procedurale_valvuloplastica,
                patient.procedurale_valvuloplastica_note,
                patient.procedurale_bioprotesi_modello,
                patient.procedurale_bioprotesi_dimensione,
            ],
        );

        match result {
            Ok(_) => {
                let patient_id = conn.last_insert_rowid();

                // Inserisci in patients_da_valutare (stato iniziale)
                match conn.execute(
                    "INSERT INTO patients_da_valutare (patient_id) VALUES (?1)",
                    params![patient_id],
                ) {
                    Ok(_) => {
                        conn.execute("COMMIT", []).map_err(|e| e.to_string())?;
                        Ok(patient_id)
                    }
                    Err(e) => {
                        conn.execute("ROLLBACK", []).ok();
                        Err(e.to_string())
                    }
                }
            }
            Err(e) => {
                conn.execute("ROLLBACK", []).ok();
                Err(e.to_string())
            }
        }
    }

    /// Aggiorna anagrafica paziente (non cambia stato)
    pub fn update_patient(&self, patient: &Patient) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        let id = patient.id.ok_or("Patient ID is required for update")?;
        let fattori_json = patient.ambulatorio_fattori.as_ref().and_then(|f| serde_json::to_string(f).ok());

        conn.execute(
            "UPDATE patients SET
                nome = ?1, cognome = ?2, data_nascita = ?3,
                luogo_nascita = ?4, codice_fiscale = ?5, telefono = ?6,
                email = ?7, provenienza = ?8, sesso = ?9, altezza = ?10, peso = ?11,
                note = ?12,
                ambulatorio_fattori = ?13, anamnesi_cardiologica = ?14, apr = ?15,
                visita_odierna = ?16, conclusioni = ?17, medico_titolo = ?18, medico_nome = ?19,
                medico_specializzando_titolo = ?20, medico_specializzando_nome = ?21,
                procedurale_allergia_mdc = ?22, procedurale_preparazione_mdc = ?23, procedurale_creatinina = ?24,
                procedurale_egfr = ?25, procedurale_hb = ?26, procedurale_altro = ?27, data_tavi = ?28,
                procedurale_ecg_ritmo_sinusale = ?29, procedurale_ecg_fa = ?30, procedurale_ecg_bbs = ?31,
                procedurale_ecg_bbd = ?32, procedurale_ecg_eas = ?33, procedurale_ecg_bav_primo = ?34,
                procedurale_ecg_ritmo_stimolato = ?35,
                procedurale_anestesia = ?36, procedurale_coronarografia = ?37, procedurale_coronarografia_note = ?38,
                procedurale_pacemaker = ?39, procedurale_pacemaker_note = ?40,
                procedurale_accesso_principale_fem = ?41, procedurale_accesso_principale_altro = ?42,
                procedurale_accesso_protezione = ?43, procedurale_accesso_protezione_note = ?44,
                procedurale_altri_accessi = ?45, procedurale_diametro_pallone_femorale = ?46,
                procedurale_guida_safari = ?47, procedurale_protezione_osti = ?48,
                procedurale_valvuloplastica = ?49, procedurale_valvuloplastica_note = ?50,
                procedurale_bioprotesi_modello = ?51, procedurale_bioprotesi_dimensione = ?52,
                updated_at = CURRENT_TIMESTAMP
             WHERE id = ?53",
            params![
                patient.nome,
                patient.cognome,
                patient.data_nascita,
                patient.luogo_nascita,
                patient.codice_fiscale,
                patient.telefono,
                patient.email,
                patient.provenienza,
                patient.sesso,
                patient.altezza,
                patient.peso,
                patient.note,
                fattori_json,
                patient.anamnesi_cardiologica,
                patient.apr,
                patient.visita_odierna,
                patient.conclusioni,
                patient.medico_titolo,
                patient.medico_nome,
                patient.medico_specializzando_titolo,
                patient.medico_specializzando_nome,
                patient.procedurale_allergia_mdc,
                patient.procedurale_preparazione_mdc,
                patient.procedurale_creatinina,
                patient.procedurale_egfr,
                patient.procedurale_hb,
                patient.procedurale_altro,
                patient.data_tavi,
                patient.procedurale_ecg_ritmo_sinusale,
                patient.procedurale_ecg_fa,
                patient.procedurale_ecg_bbs,
                patient.procedurale_ecg_bbd,
                patient.procedurale_ecg_eas,
                patient.procedurale_ecg_bav_primo,
                patient.procedurale_ecg_ritmo_stimolato,
                patient.procedurale_anestesia,
                patient.procedurale_coronarografia,
                patient.procedurale_coronarografia_note,
                patient.procedurale_pacemaker,
                patient.procedurale_pacemaker_note,
                patient.procedurale_accesso_principale_fem,
                patient.procedurale_accesso_principale_altro,
                patient.procedurale_accesso_protezione,
                patient.procedurale_accesso_protezione_note,
                patient.procedurale_altri_accessi,
                patient.procedurale_diametro_pallone_femorale,
                patient.procedurale_guida_safari,
                patient.procedurale_protezione_osti,
                patient.procedurale_valvuloplastica,
                patient.procedurale_valvuloplastica_note,
                patient.procedurale_bioprotesi_modello,
                patient.procedurale_bioprotesi_dimensione,
                id,
            ],
        ).map_err(|e| e.to_string())?;

        Ok(())
    }

    /// Elimina paziente (CASCADE rimuove automaticamente da tabelle stato)
    pub fn delete_patient(&self, id: i64) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM patients WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Ottieni tutti i pazienti con status
    pub fn get_all_patients_with_status(&self, filters: Option<PatientFilters>) -> Result<Vec<PatientWithStatus>, String> {
        let conn = self.conn.lock().unwrap();
        self.ensure_status_tables(&conn).map_err(|e| e.to_string())?;

        let mut query = String::from(
            "SELECT p.*, 'Da valutare' as status, dv.created_at as status_created_at
             FROM patients p INNER JOIN patients_da_valutare dv ON p.id = dv.patient_id
             UNION ALL
             SELECT p.*, 'In corso di accertamenti' as status, ae.created_at as status_created_at
             FROM patients p INNER JOIN patients_in_attesa_esami ae ON p.id = ae.patient_id
             UNION ALL
             SELECT p.*, 'In attesa di TAVI' as status, ai.created_at as status_created_at
             FROM patients p INNER JOIN patients_in_attesa_intervento ai ON p.id = ai.patient_id
             UNION ALL
             SELECT p.*, 'Non candidabile a TAVI' as status, nc.created_at as status_created_at
             FROM patients p INNER JOIN patients_non_candidabile nc ON p.id = nc.patient_id
             UNION ALL
             SELECT p.*, 'TAVI eseguita' as status, c.created_at as status_created_at
             FROM patients p INNER JOIN patients_completato c ON p.id = c.patient_id"
        );

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![];

        if let Some(filters) = filters {
            if let Some(search) = filters.search_query {
                if !search.is_empty() {
                    query = format!(
                        "SELECT * FROM ({}) WHERE nome LIKE ?1 OR cognome LIKE ?1 OR codice_fiscale LIKE ?1",
                        query
                    );
                    params.push(Box::new(format!("%{}%", search)));
                }
            }

            if let Some(status_filter) = filters.status {
                if status_filter != "all" {
                    query = format!("SELECT * FROM ({}) WHERE status = ?{}", query, params.len() + 1);
                    params.push(Box::new(status_filter));
                }
            }
        }

        query.push_str(" ORDER BY status_created_at DESC");

        let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;
        let rows = stmt.query_map(params_refs.as_slice(), |row| {
            let fattori: Option<Vec<String>> = match row.get::<_, Option<String>>("ambulatorio_fattori") {
                Ok(Some(json)) => serde_json::from_str(&json).ok(),
                Ok(None) => None,
                Err(_) => None,
            };
            Ok(PatientWithStatus {
                patient: Patient {
                    id: Some(row.get("id")?),
                    created_at: row.get("created_at").ok(),
                    updated_at: row.get("updated_at").ok(),
                    nome: row.get("nome")?,
                    cognome: row.get("cognome")?,
                    data_nascita: row.get("data_nascita")?,
                    luogo_nascita: row.get("luogo_nascita").ok(),
                    codice_fiscale: row.get("codice_fiscale").ok(),
                    telefono: row.get("telefono").ok(),
                    email: row.get("email").ok(),
                    provenienza: row.get("provenienza").ok(),
                    sesso: row.get("sesso").ok(),
                    altezza: row.get("altezza").ok(),
                    peso: row.get("peso").ok(),
                    note: row.get("note").ok(),
                    ambulatorio_fattori: fattori,
                    anamnesi_cardiologica: row.get("anamnesi_cardiologica").ok(),
                    apr: row.get("apr").ok(),
                    visita_odierna: row.get("visita_odierna").ok(),
                    conclusioni: row.get("conclusioni").ok(),
                    medico_titolo: row.get("medico_titolo").ok(),
                    medico_nome: row.get("medico_nome").ok(),
                    medico_specializzando_titolo: row.get("medico_specializzando_titolo").ok(),
                    medico_specializzando_nome: row.get("medico_specializzando_nome").ok(),
                    procedurale_allergia_mdc: row.get("procedurale_allergia_mdc").ok(),
                    procedurale_preparazione_mdc: row.get("procedurale_preparazione_mdc").ok(),
                    procedurale_creatinina: row.get("procedurale_creatinina").ok(),
                    procedurale_egfr: row.get("procedurale_egfr").ok(),
                    procedurale_hb: row.get("procedurale_hb").ok(),
                    procedurale_altro: row.get("procedurale_altro").ok(),
                    data_tavi: row.get("data_tavi").ok(),
                    procedurale_ecg_ritmo_sinusale: row.get("procedurale_ecg_ritmo_sinusale").ok(),
                    procedurale_ecg_fa: row.get("procedurale_ecg_fa").ok(),
                    procedurale_ecg_bbs: row.get("procedurale_ecg_bbs").ok(),
                    procedurale_ecg_bbd: row.get("procedurale_ecg_bbd").ok(),
                    procedurale_ecg_eas: row.get("procedurale_ecg_eas").ok(),
                    procedurale_ecg_bav_primo: row.get("procedurale_ecg_bav_primo").ok(),
                    procedurale_ecg_ritmo_stimolato: row.get("procedurale_ecg_ritmo_stimolato").ok(),
                    procedurale_anestesia: row.get("procedurale_anestesia").ok(),
                    procedurale_coronarografia: row.get("procedurale_coronarografia").ok(),
                    procedurale_coronarografia_note: row.get("procedurale_coronarografia_note").ok(),
                    procedurale_pacemaker: row.get("procedurale_pacemaker").ok(),
                    procedurale_pacemaker_note: row.get("procedurale_pacemaker_note").ok(),
                    procedurale_accesso_principale_fem: row.get("procedurale_accesso_principale_fem").ok(),
                    procedurale_accesso_principale_altro: row.get("procedurale_accesso_principale_altro").ok(),
                    procedurale_accesso_protezione: row.get("procedurale_accesso_protezione").ok(),
                    procedurale_accesso_protezione_note: row.get("procedurale_accesso_protezione_note").ok(),
                    procedurale_altri_accessi: row.get("procedurale_altri_accessi").ok(),
                    procedurale_diametro_pallone_femorale: row.get("procedurale_diametro_pallone_femorale").ok(),
                    procedurale_guida_safari: row.get("procedurale_guida_safari").ok(),
                    procedurale_protezione_osti: row.get("procedurale_protezione_osti").ok(),
                    procedurale_valvuloplastica: row.get("procedurale_valvuloplastica").ok(),
                    procedurale_valvuloplastica_note: row.get("procedurale_valvuloplastica_note").ok(),
                    procedurale_bioprotesi_modello: row.get("procedurale_bioprotesi_modello").ok(),
                    procedurale_bioprotesi_dimensione: row.get("procedurale_bioprotesi_dimensione").ok(),
                },
                status: row.get("status")?,
                status_created_at: row.get("status_created_at")?,
            })
        }).map_err(|e| e.to_string())?;

        let patients: Result<Vec<_>, _> = rows.collect();
        patients.map_err(|e| e.to_string())
    }

    /// Ottieni singolo paziente per ID
    pub fn get_patient_by_id(&self, id: i64) -> Result<Option<PatientWithStatus>, String> {
        let all_patients = self.get_all_patients_with_status(None)?;
        Ok(all_patients.into_iter().find(|p| p.patient.id == Some(id)))
    }

    /// Cambia stato paziente
    pub fn change_patient_status(&self, patient_id: i64, new_status: PatientStatus) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        self.ensure_status_tables(&conn).map_err(|e| e.to_string())?;

        conn.execute("BEGIN TRANSACTION", []).map_err(|e| e.to_string())?;

        // Rimuovi da tutte le tabelle di stato
        let tables = vec![
            "patients_da_valutare",
            "patients_in_attesa_esami",
            "patients_in_attesa_intervento",
            "patients_non_candidabile",
            "patients_completato",
        ];

        for table in tables {
            match conn.execute(
                &format!("DELETE FROM {} WHERE patient_id = ?1", table),
                params![patient_id],
            ) {
                Ok(_) => {},
                Err(e) => {
                    conn.execute("ROLLBACK", []).ok();
                    return Err(e.to_string());
                }
            }
        }

        // Inserisci nella nuova tabella
        let new_table = new_status.to_table_name();
        match conn.execute(
            &format!("INSERT INTO {} (patient_id) VALUES (?1)", new_table),
            params![patient_id],
        ) {
            Ok(_) => {
                conn.execute("COMMIT", []).map_err(|e| e.to_string())?;
                Ok(())
            }
            Err(e) => {
                conn.execute("ROLLBACK", []).ok();
                Err(e.to_string())
            }
        }
    }

    /// Ottieni contatori per stato
    pub fn get_patient_status_counts(&self) -> Result<Vec<PatientStatusCount>, String> {
        let conn = self.conn.lock().unwrap();
        self.ensure_status_tables(&conn).map_err(|e| e.to_string())?;

        let statuses = vec![
            ("Da valutare", "patients_da_valutare"),
            ("In corso di accertamenti", "patients_in_attesa_esami"),
            ("In attesa di TAVI", "patients_in_attesa_intervento"),
            ("Non candidabile a TAVI", "patients_non_candidabile"),
            ("TAVI eseguita", "patients_completato"),
        ];

        let mut counts = Vec::new();

        for (label, table) in statuses {
            let count: i32 = conn.query_row(
                &format!("SELECT COUNT(*) FROM {}", table),
                [],
                |row| row.get(0),
            ).map_err(|e| e.to_string())?;

            counts.push(PatientStatusCount {
                status: label.to_string(),
                count,
            });
        }

        Ok(counts)
    }

    /// Ottieni pazienti per stato specifico
    pub fn get_patients_by_status(&self, status: &str) -> Result<Vec<PatientWithStatus>, String> {
        let filters = PatientFilters {
            search_query: None,
            status: Some(status.to_string()),
        };
        self.get_all_patients_with_status(Some(filters))
    }
}
