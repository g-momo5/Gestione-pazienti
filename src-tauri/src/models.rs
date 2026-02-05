use serde::{Deserialize, Serialize};
use chrono::{NaiveDate, NaiveTime, Datelike};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Procedure {
    pub id: Option<i64>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,

    // DATI PAZIENTE
    pub nome: String,
    pub cognome: String,
    pub data_nascita: String,  // Format: YYYY-MM-DD
    pub altezza: Option<f64>,  // cm
    pub peso: Option<f64>,     // kg

    // DATI PRE-PROCEDURALI
    pub fe: Option<f64>,              // Frazione Eiezione %
    pub vmax: Option<f64>,            // Velocità massima m/s
    pub gmax: Option<f64>,            // Gradiente massimo mmHg
    pub gmed: Option<f64>,            // Gradiente medio mmHg
    pub ava: Option<f64>,             // Area Valvolare Aortica cm²
    pub anulus_aortico: Option<f64>,  // mm
    pub valvola_protesica: bool,
    pub protesica_modello: Option<String>,
    pub protesica_dimensione: Option<String>,

    // DATI PROCEDURALI
    pub data_procedura: String,  // Format: YYYY-MM-DD
    pub ora_inizio: String,      // Format: HH:MM
    pub ora_fine: String,        // Format: HH:MM
    pub tipo_valvola: String,    // 'Balloon Expandable' or 'Self Expandable'
    pub modello_valvola: String,
    pub dimensione_valvola: Option<f64>,  // mm
    pub pre_dilatazione: bool,
    pub post_dilatazione: bool,
}

impl Procedure {
    /// Calcola l'età del paziente in anni completi
    pub fn calculate_age(&self) -> Option<i32> {
        let birth = NaiveDate::parse_from_str(&self.data_nascita, "%Y-%m-%d").ok()?;
        let today = chrono::Local::now().date_naive();

        let mut age = today.year() - birth.year();
        if today.month() < birth.month() ||
           (today.month() == birth.month() && today.day() < birth.day()) {
            age -= 1;
        }

        Some(age)
    }

    /// Calcola il BMI (Body Mass Index)
    pub fn calculate_bmi(&self) -> Option<f64> {
        let peso = self.peso?;
        let altezza = self.altezza?;

        if altezza <= 0.0 {
            return None;
        }

        let altezza_m = altezza / 100.0;
        Some((peso / (altezza_m * altezza_m) * 10.0).round() / 10.0)
    }

    /// Calcola la durata della procedura in minuti
    pub fn calculate_duration_minutes(&self) -> Option<i32> {
        let inizio = NaiveTime::parse_from_str(&self.ora_inizio, "%H:%M").ok()?;
        let fine = NaiveTime::parse_from_str(&self.ora_fine, "%H:%M").ok()?;

        let duration = fine.signed_duration_since(inizio);
        Some(duration.num_minutes() as i32)
    }

    /// Restituisce nome completo
    pub fn full_name(&self) -> String {
        format!("{} {}", self.nome, self.cognome)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcedureFilters {
    pub search_query: Option<String>,
    pub tipo_valvola: Option<String>,  // 'all', 'Balloon Expandable', 'Self Expandable'
    pub period: Option<String>,        // 'all', '1m', '3m', '6m', '1y'
}

impl Default for ProcedureFilters {
    fn default() -> Self {
        Self {
            search_query: None,
            tipo_valvola: Some("all".to_string()),
            period: Some("all".to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Statistics {
    pub total_procedures: i32,
    pub average_duration_minutes: f64,
    pub pre_dilatazione_percentage: f64,
    pub post_dilatazione_percentage: f64,
    pub average_fe: Option<f64>,
    pub average_vmax: Option<f64>,
    pub average_gmax: Option<f64>,
    pub average_gmed: Option<f64>,
    pub average_ava: Option<f64>,
    pub balloon_expandable_count: i32,
    pub self_expandable_count: i32,
    pub top_valve_models: Vec<(String, i32)>,  // (model_name, count)
}

// ============================================================================
// PATIENT MODELS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Patient {
    pub id: Option<i64>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub nome: String,
    pub cognome: String,
    pub data_nascita: String,  // Format: YYYY-MM-DD
    pub luogo_nascita: Option<String>,
    pub codice_fiscale: Option<String>,
    pub telefono: Option<String>,
    pub email: Option<String>,
    pub provenienza: Option<String>,
    pub sesso: Option<String>,
    pub priority: Option<String>,
    pub altezza: Option<f64>,  // cm
    pub peso: Option<f64>,     // kg
    pub note: Option<String>,
    pub ambulatorio_fattori: Option<Vec<String>>,
    pub anamnesi_cardiologica: Option<String>,
    pub apr: Option<String>,
    pub visita_odierna: Option<String>,
    pub conclusioni: Option<String>,
    pub medico_titolo: Option<String>,
    pub medico_nome: Option<String>,
    pub medico_specializzando_titolo: Option<String>,
    pub medico_specializzando_nome: Option<String>,
    pub procedurale_allergia_mdc: Option<String>,
    pub procedurale_preparazione_mdc: Option<String>,
    pub procedurale_creatinina: Option<String>,
    pub procedurale_egfr: Option<String>,
    pub procedurale_hb: Option<String>,
    pub procedurale_altro: Option<String>,
    pub data_tavi: Option<String>,
    pub procedurale_ecg_ritmo_sinusale: Option<bool>,
    pub procedurale_ecg_fa: Option<bool>,
    pub procedurale_ecg_bbs: Option<bool>,
    pub procedurale_ecg_bbd: Option<bool>,
    pub procedurale_ecg_eas: Option<bool>,
    pub procedurale_ecg_bav_primo: Option<bool>,
    pub procedurale_ecg_ritmo_stimolato: Option<bool>,
    pub procedurale_anestesia: Option<String>,
    pub procedurale_coronarografia: Option<String>,
    pub procedurale_coronarografia_note: Option<String>,
    pub procedurale_pacemaker: Option<String>,
    pub procedurale_pacemaker_note: Option<String>,
    pub procedurale_accesso_principale_fem: Option<String>,
    pub procedurale_accesso_principale_altro: Option<String>,
    pub procedurale_accesso_protezione: Option<String>,
    pub procedurale_accesso_protezione_note: Option<String>,
    pub procedurale_altri_accessi: Option<String>,
    pub procedurale_diametro_pallone_femorale: Option<String>,
    pub procedurale_guida_safari: Option<String>,
    pub procedurale_protezione_osti: Option<String>,
    pub procedurale_valvuloplastica: Option<String>,
    pub procedurale_valvuloplastica_note: Option<String>,
    pub procedurale_bioprotesi_modello: Option<String>,
    pub procedurale_bioprotesi_dimensione: Option<String>,
}

impl Patient {
    pub fn full_name(&self) -> String {
        format!("{} {}", self.nome, self.cognome)
    }

    pub fn calculate_age(&self) -> Option<i32> {
        let birth = NaiveDate::parse_from_str(&self.data_nascita, "%Y-%m-%d").ok()?;
        let today = chrono::Local::now().date_naive();

        let mut age = today.year() - birth.year();
        if today.month() < birth.month() ||
           (today.month() == birth.month() && today.day() < birth.day()) {
            age -= 1;
        }

        Some(age)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PatientStatus {
    DaValutare,
    InAttesaEsami,
    InAttesaIntervento,
    NonCandidabile,
    Completato,
}

impl PatientStatus {
    pub fn to_table_name(&self) -> &'static str {
        match self {
            PatientStatus::DaValutare => "patients_da_valutare",
            PatientStatus::InAttesaEsami => "patients_in_attesa_esami",
            PatientStatus::InAttesaIntervento => "patients_in_attesa_intervento",
            PatientStatus::NonCandidabile => "patients_non_candidabile",
            PatientStatus::Completato => "patients_completato",
        }
    }

    pub fn to_label(&self) -> &'static str {
        match self {
            PatientStatus::DaValutare => "Da valutare",
            PatientStatus::InAttesaEsami => "In corso di accertamenti",
            PatientStatus::InAttesaIntervento => "In attesa di TAVI",
            PatientStatus::NonCandidabile => "Non candidabile a TAVI",
            PatientStatus::Completato => "TAVI eseguita",
        }
    }

    pub fn from_label(label: &str) -> Option<Self> {
        match label {
            "Da valutare" => Some(PatientStatus::DaValutare),
            "In corso di accertamenti" => Some(PatientStatus::InAttesaEsami),
            "In attesa di TAVI" => Some(PatientStatus::InAttesaIntervento),
            "Non candidabile a TAVI" => Some(PatientStatus::NonCandidabile),
            "TAVI eseguita" => Some(PatientStatus::Completato),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatientWithStatus {
    pub patient: Patient,
    pub status: String,
    pub status_created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatientFilters {
    pub search_query: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatientStatusCount {
    pub status: String,
    pub count: i32,
}
