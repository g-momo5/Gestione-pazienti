use crate::commands::{read_settings_from_disk, write_settings_to_disk, AppSettings};
use chrono::Utc;
use futures_util::StreamExt;
use reqwest::Client;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::fs::{create_dir_all, remove_file, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tauri::{AppHandle, Manager};

const UPDATE_MANIFEST_URL: &str =
    "https://github.com/g-momo5/Gestione-pazienti/releases/latest/download/update-manifest.json";
const UPDATE_REPO_OWNER: &str = "g-momo5";
const UPDATE_REPO_NAME: &str = "Gestione-pazienti";
const UPDATE_HTTP_USER_AGENT: &str = "gestionale-pazienti-tavi-updater";
pub const APP_UPDATE_PROGRESS_EVENT: &str = "app://update-download-progress";

const STATE_IDLE: &str = "idle";
const STATE_AVAILABLE: &str = "available";
const STATE_DOWNLOADING: &str = "downloading";
const STATE_DOWNLOADED: &str = "downloaded";
const STATE_ERROR: &str = "error";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateManifest {
    version: String,
    #[serde(default, alias = "published_at")]
    published_at: Option<String>,
    #[serde(default, alias = "body")]
    notes: Option<String>,
    assets: UpdateManifestAssets,
}

#[allow(dead_code)]
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateManifestAssets {
    #[serde(default)]
    windows: Option<ManifestAsset>,
    #[serde(default)]
    macos: Option<ManifestAsset>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum ManifestAsset {
    Url(String),
    Object(ManifestAssetObject),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ManifestAssetObject {
    url: String,
    #[serde(default)]
    file_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GithubLatestRelease {
    tag_name: String,
    #[serde(default)]
    body: Option<String>,
    #[serde(default)]
    published_at: Option<String>,
    #[serde(default)]
    assets: Vec<GithubReleaseAsset>,
}

#[derive(Debug, Deserialize)]
struct GithubReleaseAsset {
    name: String,
    browser_download_url: String,
}

impl ManifestAsset {
    fn url(&self) -> &str {
        match self {
            Self::Url(url) => url.as_str(),
            Self::Object(obj) => obj.url.as_str(),
        }
    }

    fn file_name(&self) -> Option<&str> {
        match self {
            Self::Url(_) => None,
            Self::Object(obj) => obj.file_name.as_deref(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppUpdateStatus {
    pub state: String,
    pub current_version: String,
    pub available_version: Option<String>,
    pub notes: Option<String>,
    pub published_at: Option<String>,
    pub download_url: Option<String>,
    pub installer_path: Option<String>,
    pub deferred: bool,
    pub update_available: bool,
    pub should_prompt_download: bool,
    pub should_prompt_install: bool,
    pub last_check: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdateProgressPayload {
    downloaded_bytes: u64,
    total_bytes: Option<u64>,
    percent: Option<f64>,
    version: String,
}

fn now_iso() -> String {
    Utc::now().to_rfc3339()
}

fn normalize_semver(version: &str) -> Result<Version, String> {
    Version::parse(version.trim().trim_start_matches('v'))
        .map_err(|e| format!("Versione non valida \"{version}\": {e}"))
}

fn update_cache_dir(app_handle: &AppHandle) -> PathBuf {
    let base = tauri::api::path::app_cache_dir(&app_handle.config())
        .or_else(|| tauri::api::path::app_data_dir(&app_handle.config()))
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("updates")
}

fn installer_extension() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "exe"
    }
    #[cfg(target_os = "macos")]
    {
        "dmg"
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        "bin"
    }
}

fn sanitize_filename(input: &str) -> String {
    input
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_') {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn infer_asset_filename(asset: &ManifestAsset, version: &str) -> String {
    if let Some(name) = asset.file_name() {
        let sanitized = sanitize_filename(name);
        if !sanitized.is_empty() {
            return sanitized;
        }
    }

    let base_from_url = asset
        .url()
        .split('?')
        .next()
        .and_then(|u| u.rsplit('/').next())
        .map(sanitize_filename)
        .filter(|name| !name.is_empty());

    base_from_url.unwrap_or_else(|| format!("update-{version}.{}", installer_extension()))
}

fn resolve_target_asset(manifest: &UpdateManifest) -> Result<ManifestAsset, String> {
    #[cfg(target_os = "windows")]
    {
        return manifest
            .assets
            .windows
            .clone()
            .ok_or_else(|| "Asset updater Windows mancante nel manifest".to_string());
    }
    #[cfg(target_os = "macos")]
    {
        return manifest
            .assets
            .macos
            .clone()
            .ok_or_else(|| "Asset updater macOS mancante nel manifest".to_string());
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        Err("Updater non supportato su questo sistema".to_string())
    }
}

fn clear_pending_update(settings: &mut AppSettings) {
    if let Some(installer_path) = settings.update_installer_path.as_deref() {
        let path = Path::new(installer_path);
        if path.exists() {
            let _ = remove_file(path);
        }
    }

    settings.update_state = Some(STATE_IDLE.to_string());
    settings.update_version = None;
    settings.update_notes = None;
    settings.update_published_at = None;
    settings.update_download_url = None;
    settings.update_installer_path = None;
    settings.update_deferred = Some(false);
}

fn status_from_settings(
    current_version: String,
    settings: &AppSettings,
    runtime_error: Option<String>,
) -> AppUpdateStatus {
    let state = settings
        .update_state
        .clone()
        .unwrap_or_else(|| STATE_IDLE.to_string());
    let deferred = settings.update_deferred.unwrap_or(false);
    let update_available = matches!(
        state.as_str(),
        STATE_AVAILABLE | STATE_DOWNLOADING | STATE_DOWNLOADED
    );

    AppUpdateStatus {
        state: state.clone(),
        current_version,
        available_version: settings.update_version.clone(),
        notes: settings.update_notes.clone(),
        published_at: settings.update_published_at.clone(),
        download_url: settings.update_download_url.clone(),
        installer_path: settings.update_installer_path.clone(),
        deferred,
        update_available,
        should_prompt_download: state == STATE_AVAILABLE && !deferred,
        should_prompt_install: state == STATE_DOWNLOADED && !deferred,
        last_check: settings.update_last_check.clone(),
        last_error: runtime_error.or_else(|| settings.update_last_error.clone()),
    }
}

async fn fetch_manifest() -> Result<UpdateManifest, String> {
    let client = Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(|e| format!("Impossibile inizializzare il client HTTP: {e}"))?;

    match fetch_manifest_from_static(&client).await {
        Ok(manifest) => Ok(manifest),
        Err(primary_error) => match fetch_manifest_from_github_latest(&client).await {
            Ok(manifest) => Ok(manifest),
            Err(fallback_error) => {
                if primary_error.contains("HTTP 404") && fallback_error.contains("HTTP 404") {
                    return Err(
                        "Nessuna release stabile pubblica trovata su GitHub (oppure repository non accessibile). \
Pubblica una release con almeno un asset .exe (Windows) o .dmg (macOS)."
                            .to_string(),
                    );
                }
                Err(format!(
                    "{primary_error}. Fallback API GitHub fallito: {fallback_error}"
                ))
            }
        },
    }
}

async fn fetch_manifest_from_static(client: &Client) -> Result<UpdateManifest, String> {
    let response = client
        .get(UPDATE_MANIFEST_URL)
        .header("Accept", "application/json")
        .header("User-Agent", UPDATE_HTTP_USER_AGENT)
        .send()
        .await
        .map_err(|e| format!("Errore rete durante il check aggiornamenti: {e}"))?;

    if !response.status().is_success() {
        return Err(format!(
            "Manifest aggiornamenti non disponibile (HTTP {})",
            response.status()
        ));
    }

    response
        .json::<UpdateManifest>()
        .await
        .map_err(|e| format!("Manifest aggiornamenti non valido: {e}"))
}

async fn fetch_manifest_from_github_latest(client: &Client) -> Result<UpdateManifest, String> {
    let latest_release_url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        UPDATE_REPO_OWNER, UPDATE_REPO_NAME
    );

    let response = client
        .get(latest_release_url)
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", UPDATE_HTTP_USER_AGENT)
        .send()
        .await
        .map_err(|e| format!("Errore rete interrogando GitHub releases/latest: {e}"))?;

    if !response.status().is_success() {
        return Err(format!(
            "GitHub releases/latest non disponibile (HTTP {})",
            response.status()
        ));
    }

    let release = response
        .json::<GithubLatestRelease>()
        .await
        .map_err(|e| format!("Risposta releases/latest non valida: {e}"))?;

    let version = release.tag_name.trim().trim_start_matches('v').to_string();
    if version.is_empty() {
        return Err("Tag release GitHub vuoto".to_string());
    }

    let mut windows = None;
    let mut macos = None;
    for asset in release.assets {
        let name = asset.name.to_ascii_lowercase();
        if windows.is_none() && name.ends_with(".exe") {
            windows = Some(ManifestAsset::Object(ManifestAssetObject {
                url: asset.browser_download_url.clone(),
                file_name: Some(asset.name.clone()),
            }));
        }
        if macos.is_none() && name.ends_with(".dmg") {
            macos = Some(ManifestAsset::Object(ManifestAssetObject {
                url: asset.browser_download_url.clone(),
                file_name: Some(asset.name.clone()),
            }));
        }
    }

    if windows.is_none() && macos.is_none() {
        return Err("Nessun asset .exe o .dmg trovato nell'ultima release GitHub".to_string());
    }

    Ok(UpdateManifest {
        version,
        published_at: release.published_at,
        notes: release.body,
        assets: UpdateManifestAssets { windows, macos },
    })
}

fn apply_manifest_to_settings(
    current_version: &str,
    manifest: &UpdateManifest,
    settings: &mut AppSettings,
) -> Result<(), String> {
    let current = normalize_semver(current_version)?;
    let latest = normalize_semver(&manifest.version)?;

    if latest <= current {
        clear_pending_update(settings);
        settings.update_last_error = None;
        return Ok(());
    }

    let asset = resolve_target_asset(manifest)?;
    let incoming_version = latest.to_string();
    let same_version = settings.update_version.as_deref() == Some(incoming_version.as_str());

    if !same_version {
        if let Some(old_path) = settings.update_installer_path.as_deref() {
            let path = Path::new(old_path);
            if path.exists() {
                let _ = remove_file(path);
            }
        }
        settings.update_state = Some(STATE_AVAILABLE.to_string());
        settings.update_deferred = Some(false);
        settings.update_installer_path = None;
    }

    if settings.update_state.as_deref() == Some(STATE_DOWNLOADING) {
        // Dopo un riavvio non possiamo riprendere chunk parziali: torniamo a "available".
        settings.update_state = Some(STATE_AVAILABLE.to_string());
    }

    if settings.update_state.as_deref() == Some(STATE_DOWNLOADED) {
        let installer_exists = settings
            .update_installer_path
            .as_deref()
            .map(Path::new)
            .map(|p| p.exists())
            .unwrap_or(false);
        if !installer_exists {
            settings.update_state = Some(STATE_AVAILABLE.to_string());
            settings.update_installer_path = None;
        }
    }

    if !matches!(
        settings.update_state.as_deref(),
        Some(STATE_AVAILABLE | STATE_DOWNLOADED)
    ) {
        settings.update_state = Some(STATE_AVAILABLE.to_string());
    }

    settings.update_version = Some(incoming_version);
    settings.update_notes = manifest.notes.clone();
    settings.update_published_at = manifest.published_at.clone();
    settings.update_download_url = Some(asset.url().to_string());
    settings.update_last_error = None;

    Ok(())
}

fn emit_progress(app_handle: &AppHandle, version: &str, downloaded: u64, total: Option<u64>) {
    let percent = total.map(|t| {
        if t == 0 {
            0.0
        } else {
            ((downloaded as f64 / t as f64) * 100.0).min(100.0)
        }
    });

    let payload = UpdateProgressPayload {
        downloaded_bytes: downloaded,
        total_bytes: total,
        percent,
        version: version.to_string(),
    };
    let _ = app_handle.emit_all(APP_UPDATE_PROGRESS_EVENT, payload);
}

fn persist_or_error(settings: &AppSettings) -> Result<(), String> {
    write_settings_to_disk(settings).map_err(|e| format!("Impossibile salvare stato updater: {e}"))
}

fn fail_download_update(
    settings: &mut AppSettings,
    installer_path: &Path,
    message: String,
) -> Result<AppUpdateStatus, String> {
    settings.update_state = Some(STATE_AVAILABLE.to_string());
    settings.update_last_error = Some(message.clone());
    settings.update_last_check = Some(now_iso());
    settings.update_installer_path = None;
    let _ = persist_or_error(settings);
    if installer_path.exists() {
        let _ = remove_file(installer_path);
    }
    Err(message)
}

#[tauri::command]
pub async fn check_app_update(app_handle: AppHandle) -> Result<AppUpdateStatus, String> {
    let current_version = app_handle.package_info().version.to_string();
    let mut settings = read_settings_from_disk().unwrap_or_default();
    settings.update_last_check = Some(now_iso());

    let mut runtime_error = None;

    match fetch_manifest().await {
        Ok(manifest) => {
            if let Err(err) = apply_manifest_to_settings(&current_version, &manifest, &mut settings)
            {
                settings.update_state = Some(STATE_ERROR.to_string());
                settings.update_last_error = Some(err.clone());
                runtime_error = Some(err);
            }
        }
        Err(err) => {
            settings.update_last_error = Some(err.clone());
            if settings.update_state.is_none() {
                settings.update_state = Some(STATE_IDLE.to_string());
            }
            runtime_error = Some(err);
        }
    }

    persist_or_error(&settings)?;
    Ok(status_from_settings(
        current_version,
        &settings,
        runtime_error,
    ))
}

#[tauri::command]
pub async fn download_app_update(app_handle: AppHandle) -> Result<AppUpdateStatus, String> {
    let current_version = app_handle.package_info().version.to_string();
    let mut settings = read_settings_from_disk().unwrap_or_default();

    let state = settings
        .update_state
        .clone()
        .unwrap_or_else(|| STATE_IDLE.to_string());
    let mut version = settings
        .update_version
        .clone()
        .ok_or_else(|| "Nessun aggiornamento disponibile da scaricare".to_string())?;
    let mut download_url = settings
        .update_download_url
        .clone()
        .ok_or_else(|| "URL aggiornamento mancante".to_string())?;

    if state == STATE_DOWNLOADED {
        let installer_exists = settings
            .update_installer_path
            .as_deref()
            .map(Path::new)
            .map(|p| p.exists())
            .unwrap_or(false);
        if installer_exists {
            settings.update_deferred = Some(false);
            persist_or_error(&settings)?;
            return Ok(status_from_settings(current_version, &settings, None));
        }
    }

    if !matches!(
        state.as_str(),
        STATE_AVAILABLE | STATE_ERROR | STATE_DOWNLOADED
    ) {
        return Err("Download aggiornamento non disponibile nello stato attuale".to_string());
    }

    settings.update_state = Some(STATE_DOWNLOADING.to_string());
    settings.update_deferred = Some(false);
    settings.update_last_error = None;
    settings.update_last_check = Some(now_iso());
    persist_or_error(&settings)?;

    let cache_dir = update_cache_dir(&app_handle);
    create_dir_all(&cache_dir)
        .map_err(|e| format!("Impossibile creare cartella cache updater: {e}"))?;
    let file_name =
        infer_asset_filename(&ManifestAsset::Url(download_url.clone()), version.as_str());
    let installer_path = cache_dir.join(file_name);

    let client = match Client::builder().timeout(Duration::from_secs(600)).build() {
        Ok(c) => c,
        Err(e) => {
            return fail_download_update(
                &mut settings,
                &installer_path,
                format!("Impossibile inizializzare il client download: {e}"),
            )
        }
    };

    let response = match client
        .get(&download_url)
        .header("Accept", "application/octet-stream")
        .header("User-Agent", UPDATE_HTTP_USER_AGENT)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            return fail_download_update(
                &mut settings,
                &installer_path,
                format!("Errore durante il download aggiornamento: {e}"),
            )
        }
    };

    let response = if response.status() == reqwest::StatusCode::NOT_FOUND {
        // Fallback robusto: se il link nel manifest è obsoleto/sbagliato, rilegge asset da releases/latest.
        if let Ok(latest_manifest) = fetch_manifest_from_github_latest(&client).await {
            if apply_manifest_to_settings(&current_version, &latest_manifest, &mut settings).is_ok() {
                if let Some(refreshed_url) = settings.update_download_url.clone() {
                    if refreshed_url != download_url {
                        download_url = refreshed_url;
                        version = settings.update_version.clone().unwrap_or(version);
                        settings.update_state = Some(STATE_DOWNLOADING.to_string());
                        settings.update_deferred = Some(false);
                        settings.update_last_error = None;
                        settings.update_last_check = Some(now_iso());
                        if let Err(e) = persist_or_error(&settings) {
                            return fail_download_update(&mut settings, &installer_path, e);
                        }

                        match client
                            .get(&download_url)
                            .header("Accept", "application/octet-stream")
                            .header("User-Agent", UPDATE_HTTP_USER_AGENT)
                            .send()
                            .await
                        {
                            Ok(r) => r,
                            Err(e) => {
                                return fail_download_update(
                                    &mut settings,
                                    &installer_path,
                                    format!("Errore durante il download aggiornamento (retry): {e}"),
                                )
                            }
                        }
                    } else {
                        response
                    }
                } else {
                    response
                }
            } else {
                response
            }
        } else {
            response
        }
    } else {
        response
    };

    if !response.status().is_success() {
        return fail_download_update(
            &mut settings,
            &installer_path,
            format!("Download aggiornamento fallito (HTTP {})", response.status()),
        );
    }

    let total = response.content_length();
    let mut file = match File::create(&installer_path) {
        Ok(f) => f,
        Err(e) => {
            return fail_download_update(
                &mut settings,
                &installer_path,
                format!("Impossibile creare file aggiornamento locale: {e}"),
            )
        }
    };

    emit_progress(&app_handle, &version, 0, total);

    let mut stream = response.bytes_stream();
    let mut downloaded: u64 = 0;

    while let Some(chunk) = stream.next().await {
        let chunk = match chunk {
            Ok(c) => c,
            Err(e) => {
                return fail_download_update(
                    &mut settings,
                    &installer_path,
                    format!("Errore stream download aggiornamento: {e}"),
                )
            }
        };

        if let Err(e) = file.write_all(&chunk) {
            return fail_download_update(
                &mut settings,
                &installer_path,
                format!("Errore scrittura file aggiornamento: {e}"),
            );
        }

        downloaded += chunk.len() as u64;
        emit_progress(&app_handle, &version, downloaded, total);
    }

    if let Err(e) = file.flush() {
        return fail_download_update(
            &mut settings,
            &installer_path,
            format!("Errore finalizzazione file aggiornamento: {e}"),
        );
    }

    settings.update_state = Some(STATE_DOWNLOADED.to_string());
    settings.update_installer_path = Some(installer_path.to_string_lossy().to_string());
    settings.update_deferred = Some(false);
    settings.update_last_error = None;
    settings.update_last_check = Some(now_iso());
    persist_or_error(&settings)?;

    Ok(status_from_settings(current_version, &settings, None))
}

#[tauri::command]
pub async fn install_app_update(app_handle: AppHandle) -> Result<AppUpdateStatus, String> {
    let current_version = app_handle.package_info().version.to_string();
    let mut settings = read_settings_from_disk().unwrap_or_default();
    let state = settings
        .update_state
        .clone()
        .unwrap_or_else(|| STATE_IDLE.to_string());

    if state != STATE_DOWNLOADED {
        return Err("Nessun aggiornamento scaricato da installare".to_string());
    }

    let installer_path = settings
        .update_installer_path
        .clone()
        .ok_or_else(|| "Percorso installer mancante".to_string())?;

    if !Path::new(&installer_path).exists() {
        settings.update_state = Some(STATE_AVAILABLE.to_string());
        settings.update_installer_path = None;
        settings.update_last_error =
            Some("Installer locale non trovato: riscarica aggiornamento".to_string());
        settings.update_last_check = Some(now_iso());
        persist_or_error(&settings)?;
        return Err("Installer locale non trovato, riscarica aggiornamento".to_string());
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new(&installer_path)
            .spawn()
            .map_err(|e| format!("Impossibile avviare installer Windows: {e}"))?;
        settings.update_deferred = Some(false);
        settings.update_last_error = None;
        settings.update_last_check = Some(now_iso());
        persist_or_error(&settings)?;
        let status = status_from_settings(current_version, &settings, None);
        app_handle.exit(0);
        return Ok(status);
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&installer_path)
            .spawn()
            .map_err(|e| format!("Impossibile aprire installer macOS: {e}"))?;
        settings.update_deferred = Some(false);
        settings.update_last_error = None;
        settings.update_last_check = Some(now_iso());
        persist_or_error(&settings)?;
        return Ok(status_from_settings(current_version, &settings, None));
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        Err("Installazione aggiornamento non supportata su questo sistema".to_string())
    }
}

#[tauri::command]
pub async fn dismiss_app_update(app_handle: AppHandle) -> Result<AppUpdateStatus, String> {
    let current_version = app_handle.package_info().version.to_string();
    let mut settings = read_settings_from_disk().unwrap_or_default();
    let state = settings
        .update_state
        .clone()
        .unwrap_or_else(|| STATE_IDLE.to_string());

    if matches!(state.as_str(), STATE_AVAILABLE | STATE_DOWNLOADED) {
        settings.update_deferred = Some(true);
        settings.update_last_check = Some(now_iso());
        settings.update_last_error = None;
        persist_or_error(&settings)?;
    }

    Ok(status_from_settings(current_version, &settings, None))
}
