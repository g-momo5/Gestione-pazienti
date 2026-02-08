use crate::database::Database;
use crate::models::{
    Procedure, ProcedureFilters, Statistics, Patient, PatientFilters, PatientStatus,
    PatientStatusCount, PatientWithStatus,
};
use chrono::Local;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{create_dir_all, File};
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::time::Duration;
use tauri::{AppHandle, Config, State, Window};
use tauri::Manager;
use zip::write::FileOptions;
use zip::ZipArchive;

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub db_path: Option<String>,
    pub backup_path: Option<String>,
    pub referti_amb_path: Option<String>,
    pub referti_proc_path: Option<String>,
    pub naming_amb: Option<String>,
    pub naming_proc: Option<String>,
    pub auto_open_referti: Option<bool>,
}

pub fn settings_file_path() -> PathBuf {
    let dir = tauri::api::path::app_config_dir(&Config::default())
        .unwrap_or_else(|| PathBuf::from("."));
    let _ = std::fs::create_dir_all(&dir);
    dir.join("settings.json")
}

pub fn read_settings_from_disk() -> Result<AppSettings, String> {
    let path = settings_file_path();
    if !path.exists() {
        return Ok(AppSettings::default());
    }
    let mut file = File::open(&path).map_err(|e| e.to_string())?;
    let mut buf = String::new();
    file.read_to_string(&mut buf).map_err(|e| e.to_string())?;
    let parsed: AppSettings = serde_json::from_str(&buf).map_err(|e| e.to_string())?;
    Ok(parsed)
}

pub fn write_settings_to_disk(settings: &AppSettings) -> Result<(), String> {
    let path = settings_file_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let json = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    let mut file = File::create(&path).map_err(|e| e.to_string())?;
    file.write_all(json.as_bytes()).map_err(|e| e.to_string())
}

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !dst.exists() {
        std::fs::create_dir_all(dst)?;
    }
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let dest_path = dst.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_all(&entry.path(), &dest_path)?;
        } else {
            std::fs::copy(entry.path(), &dest_path)?;
        }
    }
    Ok(())
}

fn normalize_components(path: &Path) -> Vec<Component<'_>> {
    path.components().collect()
}

fn is_subpath(child: &Path, parent: &Path) -> bool {
    let child = normalize_components(child);
    let parent = normalize_components(parent);
    if parent.is_empty() {
        return false;
    }
    child.starts_with(&parent)
}

fn same_path(a: &Path, b: &Path) -> bool {
    match (std::fs::canonicalize(a), std::fs::canonicalize(b)) {
        (Ok(a), Ok(b)) => a == b,
        _ => a == b,
    }
}

fn move_file(src: &Path, dst: &Path) -> Result<(), String> {
    if let Some(parent) = dst.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Err(rename_err) = std::fs::rename(src, dst) {
        std::fs::copy(src, dst).map_err(|e| e.to_string())?;
        eprintln!("rename file failed, copied instead: {}", rename_err);
    }
    Ok(())
}

fn move_dir_contents(src: &Path, dst: &Path) -> Result<(), String> {
    if !src.exists() {
        return Ok(());
    }
    if !dst.exists() {
        std::fs::create_dir_all(dst).map_err(|e| e.to_string())?;
    }

    for entry in std::fs::read_dir(src).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();

        if is_subpath(dst, &path) {
            continue;
        }

        let dest_path = dst.join(entry.file_name());
        let file_type = entry.file_type().map_err(|e| e.to_string())?;
        if file_type.is_dir() {
            if let Err(rename_err) = std::fs::rename(&path, &dest_path) {
                copy_dir_all(&path, &dest_path).map_err(|e| e.to_string())?;
                eprintln!("rename dir failed, copied instead: {}", rename_err);
            }
        } else {
            move_file(&path, &dest_path)?;
        }
    }

    Ok(())
}

fn find_resource_file(root: &Path, filename: &str) -> Option<PathBuf> {
    if !root.exists() {
        return None;
    }
    let entries = std::fs::read_dir(root).ok()?;
    for entry in entries {
        let entry = entry.ok()?;
        let path = entry.path();
        if path.is_dir() {
            if let Some(found) = find_resource_file(&path, filename) {
                return Some(found);
            }
        } else if path
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name == filename)
            .unwrap_or(false)
        {
            return Some(path);
        }
    }
    None
}

fn resolve_template_path(app_handle: &AppHandle, filename: &str) -> Result<PathBuf, String> {
    let candidates = [
        format!("src/lib/templates/{}", filename),
        filename.to_string(),
    ];

    for rel in candidates.iter() {
        if let Some(path) = app_handle.path_resolver().resolve_resource(rel) {
            if path.exists() {
                return Ok(path);
            }
        }
    }

    let env = app_handle.env();
    if let Some(resource_dir) = tauri::api::path::resource_dir(app_handle.package_info(), &env) {
        let direct_paths = [
            resource_dir.join(&candidates[0]),
            resource_dir.join("_up_").join(&candidates[0]),
            resource_dir.join(filename),
        ];
        for path in direct_paths.iter() {
            if path.exists() {
                return Ok(path.to_path_buf());
            }
        }
        if let Some(found) = find_resource_file(&resource_dir, filename) {
            return Ok(found);
        }
    }

    Err(format!("Template referto non trovato ({})", filename))
}

fn decode_xml_entities(value: &str) -> String {
    value
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[derive(Default, Clone)]
struct StyleInfo {
    align: Option<String>,
    size: Option<f32>,
    bold: bool,
    italic: bool,
    underline: bool,
}

fn parse_styles(styles_xml: &str) -> HashMap<String, StyleInfo> {
    let style_re = Regex::new(r#"(?s)<w:style[^>]*w:styleId="([^"]+)"[^>]*>(.*?)</w:style>"#)
        .unwrap();
    let ppr_re = Regex::new(r"(?s)<w:pPr[^>]*>(.*?)</w:pPr>").unwrap();
    let rpr_re = Regex::new(r"(?s)<w:rPr[^>]*>(.*?)</w:rPr>").unwrap();
    let align_re = Regex::new(r#"<w:jc[^>]*w:val="([^"]+)""#).unwrap();
    let sz_re = Regex::new(r#"<w:sz[^>]*w:val="(\d+)""#).unwrap();

    let mut styles = HashMap::new();
    for cap in style_re.captures_iter(styles_xml) {
        let style_id = cap.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
        let style_body = cap.get(2).map(|m| m.as_str()).unwrap_or("");
        let ppr_xml = ppr_re
            .captures(style_body)
            .and_then(|c| c.get(1).map(|m| m.as_str()))
            .unwrap_or("");
        let rpr_xml = rpr_re
            .captures(style_body)
            .and_then(|c| c.get(1).map(|m| m.as_str()))
            .unwrap_or("");

        let align = align_re
            .captures(ppr_xml)
            .and_then(|c| c.get(1).map(|m| m.as_str().to_string()));
        let size = sz_re
            .captures(rpr_xml)
            .and_then(|c| c.get(1))
            .and_then(|m| m.as_str().parse::<u32>().ok())
            .map(|v| (v as f32) / 2.0);
        let bold = rpr_xml.contains("<w:b")
            && !rpr_xml.contains("w:val=\"0\"")
            && !rpr_xml.contains("w:val=\"false\"");
        let italic = rpr_xml.contains("<w:i")
            && !rpr_xml.contains("w:val=\"0\"")
            && !rpr_xml.contains("w:val=\"false\"");
        let underline = rpr_xml.contains("<w:u")
            && !rpr_xml.contains("w:val=\"none\"");

        styles.insert(
            style_id,
            StyleInfo {
                align,
                size,
                bold,
                italic,
                underline,
            },
        );
    }

    styles
}

fn docx_xml_to_html(xml: &str, styles: Option<&HashMap<String, StyleInfo>>) -> String {
    let para_re = Regex::new(r"(?s)<w:p[^>]*>(.*?)</w:p>").unwrap();
    let ppr_re = Regex::new(r"(?s)<w:pPr[^>]*>(.*?)</w:pPr>").unwrap();
    let pstyle_re = Regex::new(r#"<w:pStyle[^>]*w:val="([^"]+)""#).unwrap();
    let align_re = Regex::new(r#"<w:jc[^>]*w:val="([^"]+)""#).unwrap();
    let run_re = Regex::new(r"(?s)<w:r[^>]*>(.*?)</w:r>").unwrap();
    let rpr_re = Regex::new(r"(?s)<w:rPr[^>]*>(.*?)</w:rPr>").unwrap();
    let sz_re = Regex::new(r#"<w:sz[^>]*w:val="(\d+)""#).unwrap();
    let br_re = Regex::new(r"<w:(br|cr)\\s*/>").unwrap();
    let tag_re = Regex::new(r"<[^>]+>").unwrap();

    let mut parts = Vec::new();

    for cap in para_re.captures_iter(xml) {
        let para_xml = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        let ppr_xml = ppr_re
            .captures(para_xml)
            .and_then(|c| c.get(1).map(|m| m.as_str()))
            .unwrap_or("");

        let style_id = pstyle_re
            .captures(ppr_xml)
            .and_then(|c| c.get(1).map(|m| m.as_str().to_string()));
        let style_info = style_id
            .as_ref()
            .and_then(|id| styles.and_then(|map| map.get(id)));

        let align = align_re
            .captures(ppr_xml)
            .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
            .or_else(|| style_info.and_then(|info| info.align.clone()))
            .unwrap_or_else(|| "left".to_string());
        let align_css = match align {
            ref v if v == "both" => "justify",
            ref v if v == "center" => "center",
            ref v if v == "right" => "right",
            _ => "left",
        };

        let p_size = sz_re
            .captures(ppr_xml)
            .and_then(|c| c.get(1))
            .and_then(|m| m.as_str().parse::<u32>().ok())
            .map(|v| (v as f32) / 2.0)
            .or_else(|| style_info.and_then(|info| info.size));

        let mut run_html = String::new();
        for run_cap in run_re.captures_iter(para_xml) {
            let run_xml = run_cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let rpr_xml = rpr_re
                .captures(run_xml)
                .and_then(|c| c.get(1).map(|m| m.as_str()))
                .unwrap_or("");

            let bold = rpr_xml.contains("<w:b")
                && !rpr_xml.contains("w:val=\"0\"")
                && !rpr_xml.contains("w:val=\"false\"");
            let italic = rpr_xml.contains("<w:i")
                && !rpr_xml.contains("w:val=\"0\"")
                && !rpr_xml.contains("w:val=\"false\"");
            let underline = rpr_xml.contains("<w:u")
                && !rpr_xml.contains("w:val=\"none\"");
            let size = sz_re
                .captures(rpr_xml)
                .and_then(|c| c.get(1))
                .and_then(|m| m.as_str().parse::<u32>().ok())
                .map(|v| (v as f32) / 2.0);

            let mut text = run_xml.replace("<w:tab/>", "\t").replace("<w:tab />", "\t");
            text = br_re.replace_all(&text, "\n").to_string();
            text = tag_re.replace_all(&text, "").to_string();
            let decoded = decode_xml_entities(&text);
            let trimmed = decoded.trim_end_matches('\r');
            if trimmed.trim().is_empty() {
                continue;
            }

            let mut escaped = escape_html(trimmed).replace('\n', "<br/>");
            escaped = escaped.replace('\t', "&emsp;");

            let mut styles: Vec<String> = Vec::new();
            if bold {
                styles.push("font-weight: 700".to_string());
            }
            if italic {
                styles.push("font-style: italic".to_string());
            }
            if underline {
                styles.push("text-decoration: underline".to_string());
            }
            if let Some(sz) = size {
                styles.push(format!("font-size: {}pt", sz));
            }

            if styles.is_empty() {
                run_html.push_str(&escaped);
            } else {
                run_html.push_str(&format!(
                    "<span style=\"{}\">{}</span>",
                    styles.join("; "),
                    escaped
                ));
            }
        }

        let trimmed = run_html.trim();
        if trimmed.is_empty() {
            continue;
        }

        let mut p_styles: Vec<String> = Vec::new();
        if align_css != "left" {
            p_styles.push(format!("text-align: {}", align_css));
        }
        if let Some(sz) = p_size {
            p_styles.push(format!("font-size: {}pt", sz));
        }
        if let Some(info) = style_info {
            if info.bold {
                p_styles.push("font-weight: 700".to_string());
            }
            if info.italic {
                p_styles.push("font-style: italic".to_string());
            }
            if info.underline {
                p_styles.push("text-decoration: underline".to_string());
            }
        }

        if p_styles.is_empty() {
            parts.push(format!("<p>{}</p>", trimmed));
        } else {
            parts.push(format!(
                "<p style=\"{}\">{}</p>",
                p_styles.join("; "),
                trimmed
            ));
        }
    }

    parts.join("\n")
}

fn resolve_referti_dir(settings: &AppSettings, kind: &str, app_handle: &AppHandle) -> PathBuf {
    let out_dir = if kind == "amb" {
        settings
            .referti_amb_path
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                tauri::api::path::app_data_dir(&app_handle.config())
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join("referti")
            })
    } else {
        settings
            .referti_proc_path
            .as_ref()
            .map(PathBuf::from)
            .or_else(|| {
                settings
                    .referti_amb_path
                    .as_ref()
                    .map(|root| PathBuf::from(root).join("Schede procedurali"))
            })
            .unwrap_or_else(|| {
                tauri::api::path::app_data_dir(&app_handle.config())
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join("referti")
            })
    };
    let _ = std::fs::create_dir_all(&out_dir);
    out_dir
}

fn resolve_moduli_temp_dir() -> PathBuf {
    let mut out_dir = std::env::temp_dir();
    out_dir.push("tavi_moduli");
    let _ = std::fs::create_dir_all(&out_dir);
    out_dir
}

fn schedule_temp_cleanup(path: PathBuf) {
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(600));
        let _ = std::fs::remove_file(&path);
    });
}

#[tauri::command]
pub async fn load_settings() -> Result<AppSettings, String> {
    read_settings_from_disk()
}

#[tauri::command]
pub async fn save_settings(settings: AppSettings, app_handle: AppHandle) -> Result<(), String> {
    let old = read_settings_from_disk().unwrap_or_default();
    let app_config = app_handle.config().clone();
    let app_data_dir = tauri::api::path::app_data_dir(&app_config)
        .unwrap_or_else(|| PathBuf::from("."));
    let default_db = app_data_dir.join("pazienti_tavi.db");
    let default_referti = app_data_dir.join("referti");

    let proc_dir_name = "Schede procedurali";

    let new_root = settings
        .referti_amb_path
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or(default_referti.clone());
    let new_db = settings
        .db_path
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| new_root.join("pazienti_tavi.db"));
    let new_proc = settings
        .referti_proc_path
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| new_root.join(proc_dir_name));

    let old_root = old
        .referti_amb_path
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or(default_referti.clone());
    let old_db = old
        .db_path
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or(default_db.clone());
    let old_proc = old
        .referti_proc_path
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or(default_referti.clone());

    let old_root_layout = old_db
        .parent()
        .map(|p| same_path(p, &old_root))
        .unwrap_or(false)
        && is_subpath(&old_proc, &old_root);
    let root_changed = !same_path(&old_root, &new_root);
    let root_overlap = is_subpath(&new_root, &old_root) || is_subpath(&old_root, &new_root);

    if old_root_layout && root_changed && old_root.exists() && !root_overlap {
        if let Some(parent) = new_root.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Err(rename_err) = std::fs::rename(&old_root, &new_root) {
            copy_dir_all(&old_root, &new_root).map_err(|e| e.to_string())?;
            eprintln!("rename root failed, copied instead: {}", rename_err);
        }

        if !same_path(&old_proc, &old_root) {
            if let Ok(rel_proc) = old_proc.strip_prefix(&old_root) {
                let moved_old_proc = new_root.join(rel_proc);
                if moved_old_proc.exists() && !same_path(&moved_old_proc, &new_proc) {
                    let _ = std::fs::create_dir_all(&new_proc);
                    move_dir_contents(&moved_old_proc, &new_proc)?;
                }
            }
        }
        let _ = std::fs::create_dir_all(&new_proc);
    } else {
        let _ = std::fs::create_dir_all(&new_root);

        if old_db.exists() && !same_path(&old_db, &new_db) {
            move_file(&old_db, &new_db)?;
        }

        if old_root.exists() && !same_path(&old_root, &new_root) {
            move_dir_contents(&old_root, &new_root)?;
        }

        if old_proc.exists()
            && !same_path(&old_proc, &new_proc)
            && !same_path(&old_proc, &old_root)
        {
            let _ = std::fs::create_dir_all(&new_proc);
            move_dir_contents(&old_proc, &new_proc)?;
        } else {
            let _ = std::fs::create_dir_all(&new_proc);
        }
    }

    write_settings_to_disk(&settings)
}

#[tauri::command]
pub async fn get_all_procedures(
    filters: Option<ProcedureFilters>,
    db: State<'_, Database>,
) -> Result<Vec<Procedure>, String> {
    db.get_all_procedures(filters)
}

#[tauri::command]
pub async fn get_procedure_by_id(id: i64, db: State<'_, Database>) -> Result<Option<Procedure>, String> {
    db.get_procedure_by_id(id)
}

#[tauri::command]
pub async fn create_procedure(procedure: Procedure, db: State<'_, Database>) -> Result<i64, String> {
    db.insert_procedure(&procedure)
}

#[tauri::command]
pub async fn update_procedure(procedure: Procedure, db: State<'_, Database>) -> Result<(), String> {
    db.update_procedure(&procedure)
}

#[tauri::command]
pub async fn delete_procedure(id: i64, db: State<'_, Database>) -> Result<(), String> {
    db.delete_procedure(id)
}

#[tauri::command]
pub async fn calculate_statistics(
    filters: Option<ProcedureFilters>,
    db: State<'_, Database>,
) -> Result<Statistics, String> {
    db.calculate_statistics(filters)
}

#[tauri::command]
pub async fn get_procedure_count(db: State<'_, Database>) -> Result<i32, String> {
    let procedures = db.get_all_procedures(None)?;
    Ok(procedures.len() as i32)
}

// ============================================================================ 
// PATIENT COMMANDS 
// ============================================================================

#[tauri::command]
pub async fn get_all_patients(
    filters: Option<PatientFilters>,
    db: State<'_, Database>,
) -> Result<Vec<PatientWithStatus>, String> {
    db.get_all_patients_with_status(filters)
}

#[tauri::command]
pub async fn get_patient_by_id(
    id: i64,
    db: State<'_, Database>,
) -> Result<Option<PatientWithStatus>, String> {
    db.get_patient_by_id(id)
}

#[tauri::command]
pub async fn create_patient(
    patient: Patient,
    db: State<'_, Database>,
) -> Result<i64, String> {
    db.insert_patient(&patient)
}

#[tauri::command]
pub async fn update_patient(
    patient: Patient,
    db: State<'_, Database>,
) -> Result<(), String> {
    db.update_patient(&patient)
}

#[tauri::command]
pub async fn delete_patient(
    id: i64,
    db: State<'_, Database>,
) -> Result<(), String> {
    db.delete_patient(id)
}

#[tauri::command]
pub async fn change_patient_status(
    patient_id: i64,
    new_status: String,
    db: State<'_, Database>,
) -> Result<(), String> {
    let status = PatientStatus::from_label(&new_status)
        .ok_or_else(|| format!("Stato non valido: {}", new_status))?;
    db.change_patient_status(patient_id, status)
}

#[tauri::command]
pub async fn get_patient_status_counts(
    db: State<'_, Database>,
) -> Result<Vec<PatientStatusCount>, String> {
    db.get_patient_status_counts()
}

#[tauri::command]
pub async fn get_patients_by_status(
    status: String,
    db: State<'_, Database>,
) -> Result<Vec<PatientWithStatus>, String> {
    db.get_patients_by_status(&status)
}

// ============================================================================
// REFERTI
// ============================================================================

fn format_date_ita(date_iso: &str) -> String {
    chrono::NaiveDate::parse_from_str(date_iso, "%Y-%m-%d")
        .map(|d| d.format("%d/%m/%Y").to_string())
        .unwrap_or_else(|_| date_iso.to_string())
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| if ['\\', '/', ':', '*', '?', '"', '<', '>', '|'].contains(&c) { '_' } else { c })
        .collect()
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    if let Some(first) = chars.next() {
        first.to_uppercase().collect::<String>() + chars.as_str()
    } else {
        String::new()
    }
}

fn replace_placeholders(content: &str, map: &HashMap<&str, String>) -> String {
    // Rimuove marcatori di controllo ortografico e normalizza i placeholder spezzati da tag.
    let proof_err_re = Regex::new(r"<w:proofErr[^>]*/>").unwrap();
    let tag_re = Regex::new(r"<[^>]+>").unwrap();
    let placeholder_clean_re = Regex::new(r"\{([^}]*)\}").unwrap();

    let mut out = proof_err_re.replace_all(content, "").to_string();
    out = placeholder_clean_re
        .replace_all(&out, |caps: &regex::Captures| {
            let inner = &caps[1];
            let cleaned = tag_re.replace_all(inner, "");
            format!("{{{}}}", cleaned)
        })
        .to_string();
    for (key, value) in map {
        // Escapa XML e preserva gli a-capo trasformandoli in <w:br/> per Word.
        let escaped = value
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;");
        let normalized = escaped
            .replace("\r\n", "<w:br/>")
            .replace('\n', "<w:br/>")
            .replace("&lt;w:br/&gt;", "<w:br/>"); // ripristina i break

        let placeholder = format!("{{{}}}", key);
        out = out.replace(&placeholder, &normalized);

        // Alcuni placeholder possono essere rimasti senza parentesi graffe nel template (es. fdrcv).
        let tag_placeholder = format!(">{}<", key);
        if out.contains(&tag_placeholder) {
            let replacement = format!(">{}<", normalized);
            out = out.replace(&tag_placeholder, &replacement);
        }
    }
    out
}

fn apply_checkbox_flags(content: &str, flags: &[bool]) -> String {
    let cb_re = Regex::new(r"<w:checkBox>.*?</w:checkBox>").unwrap();
    let default_re = Regex::new(r#"<w:default w:val="([01])"\s*/>"#).unwrap();
    let checked_re = Regex::new(r#"<w:checked w:val="([01])"\s*/>"#).unwrap();

    let mut idx = 0usize;
    cb_re
        .replace_all(content, |caps: &regex::Captures| {
            let mut frag = caps[0].to_string();
            if let Some(flag) = flags.get(idx) {
                if *flag {
                    frag = default_re
                        .replace_all(&frag, r#"<w:default w:val="1"/>"#)
                        .to_string();
                    if !frag.contains("w:default") {
                        frag = frag.replace(
                            "<w:checkBox>",
                            "<w:checkBox><w:default w:val=\"1\"/>",
                        );
                    }
                    if checked_re.is_match(&frag) {
                        frag = checked_re
                            .replace_all(&frag, r#"<w:checked w:val="1"/>"#)
                            .to_string();
                    } else {
                        frag = frag.replace(
                            "<w:default w:val=\"1\"/>",
                            "<w:default w:val=\"1\"/><w:checked w:val=\"1\"/>",
                        );
                    }
                } else {
                    frag = default_re
                        .replace_all(&frag, r#"<w:default w:val="0"/>"#)
                        .to_string();
                    frag = checked_re
                        .replace_all(&frag, r#"<w:checked w:val="0"/>"#)
                        .to_string();
                }
            }
            idx += 1;
            frag
        })
        .to_string()
}

fn title_case(value: &str) -> String {
    value
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[tauri::command]
pub async fn generate_ambulatorio_referto(
    patient_id: i64,
    db: State<'_, Database>,
    app_handle: AppHandle,
) -> Result<String, String> {
    let patient = db
        .get_patient_by_id(patient_id)?
        .ok_or_else(|| "Paziente non trovato".to_string())?;

    let p = patient.patient;
    let visit_date = Local::now().format("%d/%m/%Y").to_string();

    let sig_sigra = match p.sesso.as_deref() {
        Some("F") | Some("f") => "Sig.ra".to_string(),
        _ => "Sig.".to_string(),
    };

    let drdrssa = match p.medico_titolo.as_deref() {
        Some(t) if t.to_lowercase().contains("ssa") => "Dott.ssa".to_string(),
        _ => "Dott.".to_string(),
    };

    let drdrssasp = match p.medico_specializzando_titolo.as_deref() {
        Some(t) if t.to_lowercase().contains("ssa") => "Dott.ssa".to_string(),
        _ => "Dott.".to_string(),
    };
    let specializzando_name = p.medico_specializzando_nome.unwrap_or_default();
    let has_specializzando = !specializzando_name.trim().is_empty();
    let specializzando_label = if has_specializzando {
        "Il Medico in formazione specialistica,".to_string()
    } else {
        String::new()
    };

    let fattori_raw = p
        .ambulatorio_fattori
        .clone()
        .unwrap_or_default()
        .join(", ");
    let h_fattori = if fattori_raw.is_empty() {
        String::new()
    } else {
        "Fattori di rischio CV".to_string()
    };
    let fattori = capitalize_first(&fattori_raw);
    let anamnesi_cardiologica_raw = p.anamnesi_cardiologica.unwrap_or_default();
    let anamnesi_cardiologica = anamnesi_cardiologica_raw.clone(); // mantieni formattazione e a-capo
    let h_anamnesi_cardiologica = if anamnesi_cardiologica_raw.trim().is_empty() {
        String::new()
    } else {
        "Anamnesi Patologica Remota".to_string()
    };
    let apr_raw = p.apr.unwrap_or_default();
    let apr = apr_raw.clone(); // mantieni esattamente il testo inserito
    let h_apr = if apr_raw.trim().is_empty() {
        String::new()
    } else {
        "Terapia domiciliare".to_string()
    };
    let visita_raw = p.visita_odierna.unwrap_or_default();
    let visita_odierna = visita_raw.replace('-', "").trim().to_string();
    let h_visita_odierna = if visita_odierna.is_empty() {
        String::new()
    } else {
        "Valutazione Odierna".to_string()
    };
    let conclusioni_raw = p.conclusioni.unwrap_or_default();
    let conclusioni = conclusioni_raw.replace('-', "").trim().to_string();
    let h_conclusioni = if conclusioni.is_empty() {
        String::new()
    } else {
        "Conclusioni".to_string()
    };

    let replacements: HashMap<&str, String> = HashMap::from([
        ("data_visita", visit_date),
        ("sig_sigra", sig_sigra),
        ("nome", p.nome.clone()),
        ("cognome", p.cognome.clone()),
        ("dn", format_date_ita(&p.data_nascita)),
        ("cf", p.codice_fiscale.unwrap_or_default()),
        ("h_fdrcv", h_fattori),
        ("fdrcv", fattori),
        ("h_anamnesi_patologica_remota", h_anamnesi_cardiologica),
        ("anamnesi_patologica_remota", anamnesi_cardiologica),
        ("h_terapia_domiciliare", h_apr),
        ("terapia_domiciliare", apr),
        ("h_visita_odierna", h_visita_odierna),
        ("visita_odierna", visita_odierna),
        ("h_conclusioni", h_conclusioni),
        ("conclusioni", conclusioni),
        ("drdrssa", drdrssa),
        ("cardiologo", p.medico_nome.unwrap_or_default()),
        ("specializzando", specializzando_label),
        ("drdrssasp", if has_specializzando { drdrssasp } else { String::new() }),
        ("nome_specializzando", if has_specializzando { specializzando_name } else { String::new() }),
    ]);

    let template_path = resolve_template_path(&app_handle, "template_amb_strutturale.docx")?;

    let mut template_file =
        File::open(&template_path).map_err(|_| "Impossibile aprire il template".to_string())?;
    let mut archive =
        ZipArchive::new(&mut template_file).map_err(|_| "Template referto non valido".to_string())?;

    let mut output_bytes: Vec<u8> = Vec::new();
    {
        let mut writer = zip::ZipWriter::new(std::io::Cursor::new(&mut output_bytes));

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|_| "Errore lettura template".to_string())?;
            let name = file.name().to_string();

            if file.is_dir() {
                writer
                    .add_directory(name, FileOptions::default())
                    .map_err(|_| "Errore scrittura referto".to_string())?;
                continue;
            }

            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)
                .map_err(|_| "Errore lettura template".to_string())?;

            let options = FileOptions::default().compression_method(file.compression());

            if name.ends_with(".xml") {
                let content = String::from_utf8_lossy(&buffer).to_string();
                let replaced = replace_placeholders(&content, &replacements);
                writer
                    .start_file(name.clone(), options)
                    .map_err(|_| "Errore scrittura referto".to_string())?;
                writer
                    .write_all(replaced.as_bytes())
                    .map_err(|_| "Errore scrittura referto".to_string())?;
            } else {
                writer
                    .start_file(name.clone(), options)
                    .map_err(|_| "Errore scrittura referto".to_string())?;
                writer
                    .write_all(&buffer)
                    .map_err(|_| "Errore scrittura referto".to_string())?;
            }
        }

        writer.finish().map_err(|_| "Errore finale referto".to_string())?;
    }

    let settings = read_settings_from_disk().unwrap_or_default();
    let out_dir = resolve_referti_dir(&settings, "amb", &app_handle);
    create_dir_all(&out_dir).map_err(|_| "Impossibile creare cartella referti".to_string())?;

    let filename = sanitize_filename(&format!("{} {}.docx", p.cognome, p.nome));
    let out_path = out_dir.join(filename);

    let mut out_file =
        File::create(&out_path).map_err(|_| "Impossibile creare il referto".to_string())?;
    out_file
        .write_all(&output_bytes)
        .map_err(|_| "Errore salvataggio referto".to_string())?;

    Ok(out_path
        .to_string_lossy()
        .to_string())
}

#[tauri::command]
pub async fn generate_scheda_procedurale_referto(
    patient_id: i64,
    db: State<'_, Database>,
    app_handle: AppHandle,
) -> Result<String, String> {
    let patient = db
        .get_patient_by_id(patient_id)?
        .ok_or_else(|| "Paziente non trovato".to_string())?;

    let p = patient.patient;
    let replacements: HashMap<&str, String> = HashMap::from([
        ("nome", p.nome.clone()),
        ("cognome", p.cognome.clone()),
        ("dn", format_date_ita(&p.data_nascita)),
        ("peso", p.peso.map(|v| v.to_string()).unwrap_or_default()),
        ("altezza", p.altezza.map(|v| v.to_string()).unwrap_or_default()),
        ("creatinina", p.procedurale_creatinina.unwrap_or_default()),
        ("egfr", p.procedurale_egfr.unwrap_or_default()),
        ("hb", p.procedurale_hb.unwrap_or_default()),
        ("altro", p.procedurale_altro.unwrap_or_default()),
        (
            "modello_valvola",
            title_case(
                &p.procedurale_bioprotesi_modello
                    .unwrap_or_default()
                    .replace('_', " "),
            ),
        ),
        (
            "dimensione_valvola",
            p.procedurale_bioprotesi_dimensione
                .as_ref()
                .map(|d| format!("{} mm", d))
                .unwrap_or_default(),
        ),
        (
            "diametro_pallone_femorale",
            p.procedurale_diametro_pallone_femorale.unwrap_or_default(),
        ),
        ("guida_safari", p.procedurale_guida_safari.unwrap_or_default()),
        (
            "note_valvuloplastica",
            p.procedurale_valvuloplastica_note.unwrap_or_default(),
        ),
        ("altri_accessi", p.procedurale_altri_accessi.unwrap_or_default()),
        (
            "altro_accesso_arterioso",
            if p.procedurale_accesso_principale_fem.as_deref() == Some("altro") {
                p.procedurale_accesso_principale_altro.unwrap_or_default()
            } else {
                String::new()
            },
        ),
        (
            "note_accesso_protezione",
            p.procedurale_accesso_protezione_note.unwrap_or_default(),
        ),
        (
            "note_protezione",
            match p.procedurale_protezione_osti.as_deref() {
                Some("si") | Some("Si") | Some("SI") => "Sì".to_string(),
                Some("no") | Some("No") | Some("NO") => "No".to_string(),
                Some(other) => other.to_string(),
                None => String::new(),
            },
        ),
        (
            "note_pm_definitivo",
            p.procedurale_pacemaker_note.unwrap_or_default(),
        ),
        ("note_cvg", p.procedurale_coronarografia_note.unwrap_or_default()),
    ]);
    // Mappatura checkbox nel template in ordine:
    // 0-1 Allergia MdC (sì/no)
    // 2-8 ECG: ritmo sinusale, FA, BBS, BBD, EAS, BAV I grado, ritmo stimolato
    // 9-11 Anestesia: Locale, Sedazione, Generale
    // 12-13 Coronarografia: da eseguire in ricovero, già eseguita
    // 14-15 Pacemaker definitivo: sì/no
    // 16-20 Accesso principale: percutaneo dx, percutaneo sn, chirurgico dx, chirurgico sn, altro
    // 21-22 Accesso protezione: sì/no
    // 23-24 Protezione osti: sì/no
    // 25-26 Valvuloplastica: sì/no
    let accesso = p.procedurale_accesso_principale_fem.as_deref();
    let cb_flags: Vec<bool> = vec![
        p.procedurale_allergia_mdc.as_deref() == Some("si"),
        p.procedurale_allergia_mdc.as_deref() == Some("no"),
        p.procedurale_ecg_ritmo_sinusale.unwrap_or(false),
        p.procedurale_ecg_fa.unwrap_or(false),
        p.procedurale_ecg_bbs.unwrap_or(false),
        p.procedurale_ecg_bbd.unwrap_or(false),
        p.procedurale_ecg_eas.unwrap_or(false),
        p.procedurale_ecg_bav_primo.unwrap_or(false),
        p.procedurale_ecg_ritmo_stimolato.unwrap_or(false),
        p.procedurale_anestesia.as_deref() == Some("Locale"),
        p.procedurale_anestesia.as_deref() == Some("Sedazione"),
        p.procedurale_anestesia.as_deref() == Some("Generale"),
        p.procedurale_coronarografia.as_deref() == Some("ricovero"),
        p.procedurale_coronarografia.as_deref() == Some("gia_eseguita"),
        p.procedurale_pacemaker.as_deref() == Some("si"),
        p.procedurale_pacemaker.as_deref() == Some("no"),
        accesso == Some("percutaneo_dx"),
        accesso == Some("percutaneo_sn"),
        accesso == Some("chirurgico_dx"),
        accesso == Some("chirurgico_sn"),
        accesso == Some("altro"),
        p.procedurale_accesso_protezione.as_deref() == Some("si"),
        p.procedurale_accesso_protezione.as_deref() == Some("no"),
        matches!(p.procedurale_protezione_osti.as_deref(), Some("si") | Some("Si") | Some("SI")),
        matches!(p.procedurale_protezione_osti.as_deref(), Some("no") | Some("No") | Some("NO")),
        p.procedurale_valvuloplastica.as_deref() == Some("si"),
        p.procedurale_valvuloplastica.as_deref() == Some("no"),
    ];

    let template_path = resolve_template_path(&app_handle, "template_scheda_procedurale.docx")?;

    let mut template_file =
        File::open(&template_path).map_err(|_| "Impossibile aprire il template".to_string())?;
    let mut archive =
        ZipArchive::new(&mut template_file).map_err(|_| "Template referto non valido".to_string())?;

    let mut output_bytes: Vec<u8> = Vec::new();
    {
        let mut writer = zip::ZipWriter::new(std::io::Cursor::new(&mut output_bytes));

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|_| "Errore lettura template".to_string())?;
            let name = file.name().to_string();

            if file.is_dir() {
                writer
                    .add_directory(name, FileOptions::default())
                    .map_err(|_| "Errore scrittura referto".to_string())?;
                continue;
            }

            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)
                .map_err(|_| "Errore lettura template".to_string())?;

            let options = FileOptions::default().compression_method(file.compression());

            if name.ends_with(".xml") {
                let content = String::from_utf8_lossy(&buffer).to_string();
                let replaced = replace_placeholders(&content, &replacements);
                let with_checks = apply_checkbox_flags(&replaced, &cb_flags);
                writer
                    .start_file(name.clone(), options)
                    .map_err(|_| "Errore scrittura referto".to_string())?;
                writer
                    .write_all(with_checks.as_bytes())
                    .map_err(|_| "Errore scrittura referto".to_string())?;
            } else {
                writer
                    .start_file(name.clone(), options)
                    .map_err(|_| "Errore scrittura referto".to_string())?;
                writer
                    .write_all(&buffer)
                    .map_err(|_| "Errore scrittura referto".to_string())?;
            }
        }

        writer.finish().map_err(|_| "Errore finale referto".to_string())?;
    }

    let settings = read_settings_from_disk().unwrap_or_default();
    let out_dir = resolve_referti_dir(&settings, "proc", &app_handle);
    create_dir_all(&out_dir).map_err(|_| "Impossibile creare cartella referti".to_string())?;

    let filename = sanitize_filename(&format!("Scheda procedurale - {} {}.docx", p.cognome, p.nome));
    let out_path = out_dir.join(filename);

    let mut out_file =
        File::create(&out_path).map_err(|_| "Impossibile creare il referto".to_string())?;
    out_file
        .write_all(&output_bytes)
        .map_err(|_| "Errore salvataggio referto".to_string())?;

    Ok(out_path
        .to_string_lossy()
        .to_string())
}

#[tauri::command]
pub async fn generate_consenso_informato(
    patient_id: i64,
    db: State<'_, Database>,
    app_handle: AppHandle,
) -> Result<String, String> {
    let patient = db
        .get_patient_by_id(patient_id)?
        .ok_or_else(|| "Paziente non trovato".to_string())?;
    let p = patient.patient;

    let template_path = resolve_template_path(&app_handle, "consenso_informato_TAVI.docx")?;
    let replacements: HashMap<&str, String> = HashMap::from([
        ("nome", p.nome.clone()),
        ("cognome", p.cognome.clone()),
    ]);

    let mut template_file =
        File::open(&template_path).map_err(|_| "Impossibile aprire il template".to_string())?;
    let mut archive =
        ZipArchive::new(&mut template_file).map_err(|_| "Template referto non valido".to_string())?;

    let mut output_bytes: Vec<u8> = Vec::new();
    {
        let mut writer = zip::ZipWriter::new(std::io::Cursor::new(&mut output_bytes));
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|_| "Errore lettura template".to_string())?;
            let name = file.name().to_string();

            if file.is_dir() {
                writer
                    .add_directory(name, FileOptions::default())
                    .map_err(|_| "Errore scrittura referto".to_string())?;
                continue;
            }

            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)
                .map_err(|_| "Errore lettura template".to_string())?;

            let options = FileOptions::default().compression_method(file.compression());

            if name.ends_with(".xml") {
                let content = String::from_utf8_lossy(&buffer).to_string();
                let replaced = replace_placeholders(&content, &replacements);
                writer
                    .start_file(name.clone(), options)
                    .map_err(|_| "Errore scrittura referto".to_string())?;
                writer
                    .write_all(replaced.as_bytes())
                    .map_err(|_| "Errore scrittura referto".to_string())?;
            } else {
                writer
                    .start_file(name.clone(), options)
                    .map_err(|_| "Errore scrittura referto".to_string())?;
                writer
                    .write_all(&buffer)
                    .map_err(|_| "Errore scrittura referto".to_string())?;
            }
        }
        writer.finish().map_err(|_| "Errore finale referto".to_string())?;
    }

    let out_dir = resolve_moduli_temp_dir();
    create_dir_all(&out_dir).map_err(|_| "Impossibile creare cartella moduli".to_string())?;

    let filename = sanitize_filename(&format!(
        "Consenso informato - {} {}.docx",
        p.cognome, p.nome
    ));
    let out_path = out_dir.join(filename);

    let mut out_file =
        File::create(&out_path).map_err(|_| "Impossibile creare il referto".to_string())?;
    out_file
        .write_all(&output_bytes)
        .map_err(|_| "Errore salvataggio referto".to_string())?;

    schedule_temp_cleanup(out_path.clone());
    Ok(out_path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn generate_consenso_informato_html(
    patient_id: i64,
    db: State<'_, Database>,
    app_handle: AppHandle,
) -> Result<String, String> {
    let patient = db
        .get_patient_by_id(patient_id)?
        .ok_or_else(|| "Paziente non trovato".to_string())?;
    let p = patient.patient;

    let template_path = resolve_template_path(&app_handle, "consenso_informato_TAVI.docx")?;
    let replacements: HashMap<&str, String> = HashMap::from([
        ("nome", p.nome.clone()),
        ("cognome", p.cognome.clone()),
    ]);

    let mut template_file =
        File::open(&template_path).map_err(|_| "Impossibile aprire il template".to_string())?;
    let mut archive =
        ZipArchive::new(&mut template_file).map_err(|_| "Template referto non valido".to_string())?;

    let mut document_xml = String::new();
    let mut styles_xml = String::new();
    {
        let mut doc_file = archive
            .by_name("word/document.xml")
            .map_err(|_| "Template referto non valido".to_string())?;
        doc_file
            .read_to_string(&mut document_xml)
            .map_err(|_| "Errore lettura template".to_string())?;
    }
    if let Ok(mut styles_file) = archive.by_name("word/styles.xml") {
        let _ = styles_file.read_to_string(&mut styles_xml);
    }

    let replaced = replace_placeholders(&document_xml, &replacements);
    let styles_map = if styles_xml.is_empty() {
        HashMap::new()
    } else {
        parse_styles(&styles_xml)
    };
    let html = docx_xml_to_html(&replaced, Some(&styles_map));
    if html.trim().is_empty() {
        return Err("Contenuto consenso vuoto".to_string());
    }
    Ok(html)
}

#[tauri::command]
pub async fn generate_esami_ematochimici(
    patient_id: i64,
    db: State<'_, Database>,
    app_handle: AppHandle,
) -> Result<String, String> {
    let patient = db
        .get_patient_by_id(patient_id)?
        .ok_or_else(|| "Paziente non trovato".to_string())?;
    let p = patient.patient;

    let template_path = resolve_template_path(&app_handle, "ee_tavi.pdf")?;
    let out_dir = resolve_moduli_temp_dir();
    create_dir_all(&out_dir).map_err(|_| "Impossibile creare cartella moduli".to_string())?;

    let filename = sanitize_filename(&format!(
        "Esami ematochimici - {} {}.pdf",
        p.cognome, p.nome
    ));
    let out_path = out_dir.join(filename);
    std::fs::copy(&template_path, &out_path).map_err(|_| "Impossibile copiare il modulo".to_string())?;

    schedule_temp_cleanup(out_path.clone());
    Ok(out_path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn print_file(path: String) -> Result<(), String> {
    if path.trim().is_empty() {
        return Err("Percorso file non valido".to_string());
    }
    let target = PathBuf::from(&path);
    if !target.exists() {
        return Err("File non trovato".to_string());
    }

    #[cfg(target_os = "macos")]
    {
        let escaped = path.replace('\\', "\\\\").replace('"', "\\\"");
        let status = Command::new("osascript")
            .arg("-e")
            .arg(format!("set theFile to POSIX file \"{}\"", escaped))
            .arg("-e")
            .arg("tell application \"Finder\" to open theFile")
            .arg("-e")
            .arg("delay 1.2")
            .arg("-e")
            .arg("tell application \"System Events\" to keystroke \"p\" using command down")
            .status()
            .map_err(|e| e.to_string())?;

        if !status.success() {
            return Err("Errore apertura stampa".to_string());
        }
        return Ok(());
    }

    #[cfg(target_os = "windows")]
    {
        let script = r#"
param([string]$p)
$proc = Start-Process -FilePath $p -PassThru
Start-Sleep -Milliseconds 800
$wshell = New-Object -ComObject WScript.Shell
for ($i = 0; $i -lt 12; $i++) {
  if ($wshell.AppActivate($proc.Id)) { break }
  Start-Sleep -Milliseconds 300
}
Start-Sleep -Milliseconds 200
$wshell.SendKeys('^p')
"#;

        let status = Command::new("powershell")
            .arg("-NoProfile")
            .arg("-Command")
            .arg(script)
            .arg("-Args")
            .arg(&path)
            .status()
            .map_err(|e| e.to_string())?;

        if !status.success() {
            return Err("Errore apertura stampa".to_string());
        }
        return Ok(());
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        Err("Stampa con dialogo supportata solo su macOS e Windows".to_string())
    }
}

#[tauri::command]
pub fn print_window(window: Window) -> Result<(), String> {
    window.print().map_err(|e| e.to_string())
}
