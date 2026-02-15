use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

use crate::media;

// ─── Standard metadata structure (stored as metas/<uuid>/meta.json) ─────────

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct VideoMetadata {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overview: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tagline: Option<String>,
    #[serde(default)]
    pub genres: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_minutes: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rating: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vote_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub imdb_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tmdb_id: Option<u64>,
    #[serde(default)]
    pub cast: Vec<CastMember>,
    #[serde(default)]
    pub crew: Vec<CrewMember>,
    #[serde(default)]
    pub studios: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(default)]
    pub images: MetadataImages,
    /// Which provider was used to fetch this metadata
    pub provider: String,
    /// ISO 8601 timestamp of when this metadata was fetched
    pub fetched_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct CastMember {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub character: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_path: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct CrewMember {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub job: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_path: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct MetadataImages {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poster: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backdrop: Option<String>,
}

// ─── Filename parsing ───────────────────────────────────────────────────────

#[derive(Debug)]
struct ParsedFilename {
    title: String,
    year: Option<u32>,
}

/// Parse a video filename into a title and optional year.
/// Handles formats like:
///   "Movie.Name.2024.1080p.BluRay.x264.mkv"
///   "Movie Name (2024).mp4"
///   "Movie_Name_2024_720p.mkv"
fn parse_filename(filename: &str) -> ParsedFilename {
    // Remove extension
    let name = filename
        .rsplit_once('.')
        .map(|(name, _ext)| name)
        .unwrap_or(filename);

    // Try to extract year in parentheses: "Movie Name (2024)"
    let year_paren_re = Regex::new(r"\((\d{4})\)").unwrap();
    // Try to extract year after separators: "Movie.Name.2024.stuff"
    let year_sep_re = Regex::new(r"[\.\s_\-](\d{4})[\.\s_\-]").unwrap();
    // Year at the end: "Movie Name 2024"
    let year_end_re = Regex::new(r"[\.\s_\-](\d{4})$").unwrap();

    let mut year: Option<u32> = None;
    let mut title_end: Option<usize> = None;

    // Try parenthesized year first
    if let Some(caps) = year_paren_re.captures(name) {
        let y: u32 = caps[1].parse().unwrap_or(0);
        if (1900..=2100).contains(&y) {
            year = Some(y);
            title_end = caps.get(0).map(|m| m.start());
        }
    }

    // Try separator year
    if year.is_none() {
        if let Some(caps) = year_sep_re.captures(name) {
            let y: u32 = caps[1].parse().unwrap_or(0);
            if (1900..=2100).contains(&y) {
                year = Some(y);
                title_end = caps.get(1).map(|m| m.start());
            }
        }
    }

    // Try year at end
    if year.is_none() {
        if let Some(caps) = year_end_re.captures(name) {
            let y: u32 = caps[1].parse().unwrap_or(0);
            if (1900..=2100).contains(&y) {
                year = Some(y);
                title_end = caps.get(1).map(|m| m.start());
            }
        }
    }

    // Extract title portion
    let title_raw = if let Some(end) = title_end {
        &name[..end]
    } else {
        // Remove common quality/codec indicators
        let quality_re =
            Regex::new(r"[\.\s_\-](1080p|720p|2160p|4K|BluRay|BRRip|HDRip|WEBRip|WEB-DL|DVDRip|HDTV|CAM|TS|HC|x264|x265|H\.?264|H\.?265|HEVC|AAC|AC3|DTS|REMUX|PROPER|REPACK|EXTENDED|UNRATED|DIRECTORS\.?CUT|10bit).*$")
                .unwrap();
        if let Some(m) = quality_re.find(name) {
            &name[..m.start()]
        } else {
            name
        }
    };

    // Replace dots, underscores, hyphens with spaces and trim
    let title = title_raw
        .replace('.', " ")
        .replace('_', " ")
        .replace('-', " ")
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
        .trim()
        .to_string();

    ParsedFilename { title, year }
}

// ─── TMDB API ───────────────────────────────────────────────────────────────

const TMDB_BASE_URL: &str = "https://api.themoviedb.org/3";
const TMDB_IMAGE_BASE: &str = "https://image.tmdb.org/t/p";

#[derive(Deserialize, Debug)]
struct TmdbSearchResult {
    results: Vec<TmdbMovie>,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct TmdbMovie {
    id: u64,
    title: Option<String>,
    original_title: Option<String>,
    overview: Option<String>,
    release_date: Option<String>,
    vote_average: Option<f64>,
    vote_count: Option<u32>,
    genre_ids: Option<Vec<u32>>,
    poster_path: Option<String>,
    backdrop_path: Option<String>,
    original_language: Option<String>,
}

#[derive(Deserialize, Debug)]
struct TmdbMovieDetail {
    id: u64,
    title: Option<String>,
    original_title: Option<String>,
    overview: Option<String>,
    tagline: Option<String>,
    release_date: Option<String>,
    runtime: Option<u32>,
    vote_average: Option<f64>,
    vote_count: Option<u32>,
    genres: Option<Vec<TmdbGenre>>,
    poster_path: Option<String>,
    backdrop_path: Option<String>,
    imdb_id: Option<String>,
    original_language: Option<String>,
    status: Option<String>,
    production_companies: Option<Vec<TmdbCompany>>,
    credits: Option<TmdbCredits>,
}

#[derive(Deserialize, Debug)]
struct TmdbGenre {
    name: String,
}

#[derive(Deserialize, Debug)]
struct TmdbCompany {
    name: String,
}

#[derive(Deserialize, Debug)]
struct TmdbCredits {
    cast: Option<Vec<TmdbCast>>,
    crew: Option<Vec<TmdbCrew>>,
}

#[derive(Deserialize, Debug)]
struct TmdbCast {
    name: String,
    character: Option<String>,
    profile_path: Option<String>,
}

#[derive(Deserialize, Debug)]
struct TmdbCrew {
    name: String,
    job: Option<String>,
    profile_path: Option<String>,
}

/// Result of a TMDB fetch including image paths
struct TmdbFetchResult {
    metadata: VideoMetadata,
    poster_path: Option<String>,
    backdrop_path: Option<String>,
}

fn fetch_from_tmdb(
    client: &reqwest::blocking::Client,
    api_key: &str,
    title: &str,
    year: Option<u32>,
) -> Result<Option<TmdbFetchResult>, String> {
    // Search for the movie
    let mut url = format!(
        "{}/search/movie?api_key={}&query={}",
        TMDB_BASE_URL,
        api_key,
        urlencoded(title)
    );
    if let Some(y) = year {
        url.push_str(&format!("&year={}", y));
    }

    log::info!("[Metadata/TMDB] Searching: \"{}\" (year: {:?})", title, year);

    let resp = client
        .get(&url)
        .send()
        .map_err(|e| format!("TMDB search request failed: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("TMDB search returned status {}", resp.status()));
    }

    let search: TmdbSearchResult = resp
        .json()
        .map_err(|e| format!("Failed to parse TMDB search response: {}", e))?;

    let movie = match search.results.first() {
        Some(m) => m,
        None => {
            log::info!("[Metadata/TMDB] No results found for \"{}\"", title);
            return Ok(None);
        }
    };

    // Get detailed info with credits
    let detail_url = format!(
        "{}/movie/{}?api_key={}&append_to_response=credits",
        TMDB_BASE_URL, movie.id, api_key
    );

    let detail_resp = client
        .get(&detail_url)
        .send()
        .map_err(|e| format!("TMDB detail request failed: {}", e))?;

    if !detail_resp.status().is_success() {
        return Err(format!(
            "TMDB detail returned status {}",
            detail_resp.status()
        ));
    }

    let detail: TmdbMovieDetail = detail_resp
        .json()
        .map_err(|e| format!("Failed to parse TMDB detail response: {}", e))?;

    // Parse year from release_date
    let year = detail
        .release_date
        .as_deref()
        .and_then(|d| d.split('-').next())
        .and_then(|y| y.parse::<u32>().ok());

    // Build cast list (top 10)
    let cast = detail
        .credits
        .as_ref()
        .and_then(|c| c.cast.as_ref())
        .map(|c| {
            c.iter()
                .take(10)
                .map(|a| CastMember {
                    name: a.name.clone(),
                    character: a.character.clone(),
                    profile_path: a.profile_path.clone(),
                })
                .collect()
        })
        .unwrap_or_default();

    // Build crew list (directors, writers)
    let crew = detail
        .credits
        .as_ref()
        .and_then(|c| c.crew.as_ref())
        .map(|c| {
            c.iter()
                .filter(|m| {
                    matches!(
                        m.job.as_deref(),
                        Some("Director") | Some("Writer") | Some("Screenplay")
                    )
                })
                .map(|m| CrewMember {
                    name: m.name.clone(),
                    job: m.job.clone(),
                    profile_path: m.profile_path.clone(),
                })
                .collect()
        })
        .unwrap_or_default();

    let metadata = VideoMetadata {
        title: detail.title.unwrap_or_default(),
        original_title: detail.original_title,
        year,
        overview: detail.overview,
        tagline: detail.tagline,
        genres: detail
            .genres
            .unwrap_or_default()
            .into_iter()
            .map(|g| g.name)
            .collect(),
        runtime_minutes: detail.runtime,
        rating: detail.vote_average,
        vote_count: detail.vote_count,
        release_date: detail.release_date,
        imdb_id: detail.imdb_id,
        tmdb_id: Some(detail.id),
        cast,
        crew,
        studios: detail
            .production_companies
            .unwrap_or_default()
            .into_iter()
            .map(|c| c.name)
            .collect(),
        language: detail.original_language,
        status: detail.status,
        images: MetadataImages::default(),
        provider: "tmdb".to_string(),
        fetched_at: chrono::Local::now().to_rfc3339(),
    };

    Ok(Some(TmdbFetchResult {
        metadata,
        poster_path: detail.poster_path,
        backdrop_path: detail.backdrop_path,
    }))
}

/// Download a TMDB image and save it to the given path.
/// `size` is e.g. "w500" for poster, "w1280" for backdrop.
fn download_tmdb_image(
    client: &reqwest::blocking::Client,
    tmdb_path: &str,
    size: &str,
    save_path: &std::path::Path,
) -> Result<(), String> {
    let url = format!("{}/{}{}", TMDB_IMAGE_BASE, size, tmdb_path);
    log::info!("[Metadata/TMDB] Downloading image: {}", url);

    let resp = client
        .get(&url)
        .send()
        .map_err(|e| format!("Image download failed: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("Image download returned status {}", resp.status()));
    }

    let bytes = resp
        .bytes()
        .map_err(|e| format!("Failed to read image bytes: {}", e))?;

    fs::write(save_path, &bytes).map_err(|e| format!("Failed to save image: {}", e))?;

    Ok(())
}

// ─── OMDb API ───────────────────────────────────────────────────────────────

#[derive(Deserialize, Debug)]
struct OmdbResult {
    #[serde(rename = "Response")]
    response: String,
    #[serde(rename = "Title")]
    title: Option<String>,
    #[serde(rename = "Year")]
    year: Option<String>,
    #[serde(rename = "Plot")]
    plot: Option<String>,
    #[serde(rename = "Genre")]
    genre: Option<String>,
    #[serde(rename = "Runtime")]
    runtime: Option<String>,
    #[serde(rename = "imdbRating")]
    imdb_rating: Option<String>,
    #[serde(rename = "imdbVotes")]
    imdb_votes: Option<String>,
    #[serde(rename = "imdbID")]
    imdb_id: Option<String>,
    #[serde(rename = "Released")]
    released: Option<String>,
    #[serde(rename = "Director")]
    director: Option<String>,
    #[serde(rename = "Actors")]
    actors: Option<String>,
    #[serde(rename = "Language")]
    language: Option<String>,
    #[serde(rename = "Poster")]
    poster: Option<String>,
    #[serde(rename = "Production")]
    production: Option<String>,
}

/// Result of an OMDb fetch including poster URL
struct OmdbFetchResult {
    metadata: VideoMetadata,
    poster_url: Option<String>,
}

fn fetch_from_omdb(
    client: &reqwest::blocking::Client,
    api_key: &str,
    title: &str,
    year: Option<u32>,
) -> Result<Option<OmdbFetchResult>, String> {
    let mut url = format!(
        "https://www.omdbapi.com/?apikey={}&t={}&type=movie&plot=full",
        api_key,
        urlencoded(title)
    );
    if let Some(y) = year {
        url.push_str(&format!("&y={}", y));
    }

    log::info!("[Metadata/OMDb] Searching: \"{}\" (year: {:?})", title, year);

    let resp = client
        .get(&url)
        .send()
        .map_err(|e| format!("OMDb request failed: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("OMDb returned status {}", resp.status()));
    }

    let result: OmdbResult = resp
        .json()
        .map_err(|e| format!("Failed to parse OMDb response: {}", e))?;

    if result.response != "True" {
        log::info!("[Metadata/OMDb] No results found for \"{}\"", title);
        return Ok(None);
    }

    let year = result
        .year
        .as_deref()
        .and_then(|y| y.split('–').next()) // Handle ranges like "2019–2023"
        .and_then(|y| y.parse::<u32>().ok());

    let runtime_minutes = result.runtime.as_deref().and_then(|r| {
        r.replace(" min", "")
            .trim()
            .parse::<u32>()
            .ok()
    });

    let rating = result
        .imdb_rating
        .as_deref()
        .and_then(|r| r.parse::<f64>().ok());

    let vote_count = result
        .imdb_votes
        .as_deref()
        .map(|v| v.replace(',', ""))
        .and_then(|v| v.parse::<u32>().ok());

    let genres: Vec<String> = result
        .genre
        .as_deref()
        .map(|g| g.split(", ").map(String::from).collect())
        .unwrap_or_default();

    let cast: Vec<CastMember> = result
        .actors
        .as_deref()
        .map(|a| {
            a.split(", ")
                .map(|name| CastMember {
                    name: name.to_string(),
                    character: None,
                    profile_path: None,
                })
                .collect()
        })
        .unwrap_or_default();

    let crew: Vec<CrewMember> = result
        .director
        .as_deref()
        .map(|d| {
            d.split(", ")
                .map(|name| CrewMember {
                    name: name.to_string(),
                    job: Some("Director".to_string()),
                    profile_path: None,
                })
                .collect()
        })
        .unwrap_or_default();

    let studios: Vec<String> = result
        .production
        .as_deref()
        .filter(|p| *p != "N/A")
        .map(|p| vec![p.to_string()])
        .unwrap_or_default();

    let poster_url = result.poster.filter(|p| p != "N/A");

    let metadata = VideoMetadata {
        title: result.title.unwrap_or_default(),
        original_title: None,
        year,
        overview: result.plot,
        tagline: None,
        genres,
        runtime_minutes,
        rating,
        vote_count,
        release_date: result.released.filter(|r| r != "N/A"),
        imdb_id: result.imdb_id,
        tmdb_id: None,
        cast,
        crew,
        studios,
        language: result.language.filter(|l| l != "N/A"),
        status: None,
        images: MetadataImages::default(),
        provider: "omdb".to_string(),
        fetched_at: chrono::Local::now().to_rfc3339(),
    };

    Ok(Some(OmdbFetchResult {
        metadata,
        poster_url,
    }))
}

/// Download OMDb poster image
fn download_omdb_poster(
    client: &reqwest::blocking::Client,
    poster_url: &str,
    save_path: &std::path::Path,
) -> Result<(), String> {
    if poster_url == "N/A" || poster_url.is_empty() {
        return Ok(());
    }
    log::info!("[Metadata/OMDb] Downloading poster: {}", poster_url);

    let resp = client
        .get(poster_url)
        .send()
        .map_err(|e| format!("Poster download failed: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("Poster download returned status {}", resp.status()));
    }

    let bytes = resp
        .bytes()
        .map_err(|e| format!("Failed to read poster bytes: {}", e))?;

    fs::write(save_path, &bytes).map_err(|e| format!("Failed to save poster: {}", e))?;

    Ok(())
}

// ─── URL encoding helper ────────────────────────────────────────────────────

fn urlencoded(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            ' ' => "%20".to_string(),
            '&' => "%26".to_string(),
            '=' => "%3D".to_string(),
            '+' => "%2B".to_string(),
            '#' => "%23".to_string(),
            '?' => "%3F".to_string(),
            _ if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '~' => {
                c.to_string()
            }
            _ => format!("%{:02X}", c as u32),
        })
        .collect()
}

// ─── Metas folder management ────────────────────────────────────────────────

/// Get the metas directory path for a user
fn get_metas_dir(app: &AppHandle, user_id: &str) -> Result<PathBuf, String> {
    let user_dir = media::get_user_dir_public(app, user_id)?;
    Ok(user_dir.join("metas"))
}

/// Get the metadata directory for a specific media entry
fn get_meta_dir(app: &AppHandle, user_id: &str, media_id: &str) -> Result<PathBuf, String> {
    Ok(get_metas_dir(app, user_id)?.join(media_id))
}

/// Check if metadata exists for a given media entry
fn has_metadata(app: &AppHandle, user_id: &str, media_id: &str) -> bool {
    if let Ok(dir) = get_meta_dir(app, user_id, media_id) {
        dir.join("meta.json").exists()
    } else {
        false
    }
}

/// Get metadata for a specific media entry
pub fn get_metadata(
    app: &AppHandle,
    user_id: &str,
    media_id: &str,
) -> Result<serde_json::Value, String> {
    let meta_dir = get_meta_dir(app, user_id, media_id)?;
    let meta_path = meta_dir.join("meta.json");

    if !meta_path.exists() {
        return Err("Metadata not found".to_string());
    }

    let raw = fs::read_to_string(&meta_path).map_err(|e| e.to_string())?;
    let meta: serde_json::Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    Ok(meta)
}

/// Save metadata and download images for a media entry
fn save_metadata_and_images(
    client: &reqwest::blocking::Client,
    app: &AppHandle,
    user_id: &str,
    media_id: &str,
    metadata: &mut VideoMetadata,
    provider_id: &str,
    tmdb_poster_path: Option<&str>,
    tmdb_backdrop_path: Option<&str>,
    omdb_poster_url: Option<&str>,
) -> Result<(), String> {
    let meta_dir = get_meta_dir(app, user_id, media_id)?;
    fs::create_dir_all(&meta_dir).map_err(|e| format!("Failed to create meta dir: {}", e))?;

    // Download images based on provider
    match provider_id {
        "tmdb" => {
            // Download poster (w500 size, like Jellyfin)
            if let Some(poster) = tmdb_poster_path {
                let poster_path = meta_dir.join("poster.jpg");
                match download_tmdb_image(client, poster, "w500", &poster_path) {
                    Ok(()) => {
                        metadata.images.poster = Some("poster.jpg".to_string());
                        log::info!("[Metadata] Poster saved for {}", media_id);
                    }
                    Err(e) => log::warn!("[Metadata] Failed to download poster: {}", e),
                }
            }
            // Download backdrop (w1280 size, like Jellyfin)
            if let Some(backdrop) = tmdb_backdrop_path {
                let backdrop_path = meta_dir.join("backdrop.jpg");
                match download_tmdb_image(client, backdrop, "w1280", &backdrop_path) {
                    Ok(()) => {
                        metadata.images.backdrop = Some("backdrop.jpg".to_string());
                        log::info!("[Metadata] Backdrop saved for {}", media_id);
                    }
                    Err(e) => log::warn!("[Metadata] Failed to download backdrop: {}", e),
                }
            }
        }
        "omdb" => {
            if let Some(poster_url) = omdb_poster_url {
                let poster_path = meta_dir.join("poster.jpg");
                match download_omdb_poster(client, poster_url, &poster_path) {
                    Ok(()) => {
                        metadata.images.poster = Some("poster.jpg".to_string());
                        log::info!("[Metadata] Poster saved for {}", media_id);
                    }
                    Err(e) => log::warn!("[Metadata] Failed to download poster: {}", e),
                }
            }
        }
        _ => {}
    }

    // Save meta.json
    let meta_json =
        serde_json::to_string_pretty(metadata).map_err(|e| format!("Failed to serialize: {}", e))?;
    let meta_path = meta_dir.join("meta.json");
    fs::write(&meta_path, meta_json).map_err(|e| format!("Failed to write meta.json: {}", e))?;

    log::info!(
        "[Metadata] Metadata saved for {} (provider: {})",
        media_id,
        provider_id
    );
    Ok(())
}

// ─── Main fetch orchestration ───────────────────────────────────────────────

/// Fetch metadata for a single media entry, trying providers in order
fn fetch_metadata_for_entry(
    client: &reqwest::blocking::Client,
    app: &AppHandle,
    user_id: &str,
    media_id: &str,
    filename: &str,
    providers: &[media::MetadataProviderConfig],
) -> Result<bool, String> {
    let parsed = parse_filename(filename);
    log::info!(
        "[Metadata] Parsed filename \"{}\": title=\"{}\", year={:?}",
        filename,
        parsed.title,
        parsed.year
    );

    for provider in providers {
        if !provider.enabled || provider.api_key.is_empty() {
            continue;
        }

        match provider.id.as_str() {
            "tmdb" => {
                match fetch_from_tmdb(client, &provider.api_key, &parsed.title, parsed.year) {
                    Ok(Some(result)) => {
                        let mut metadata = result.metadata;
                        save_metadata_and_images(
                            client,
                            app,
                            user_id,
                            media_id,
                            &mut metadata,
                            "tmdb",
                            result.poster_path.as_deref(),
                            result.backdrop_path.as_deref(),
                            None,
                        )?;
                        return Ok(true);
                    }
                    Ok(None) => {
                        log::info!(
                            "[Metadata] TMDB found nothing for \"{}\", trying next provider",
                            parsed.title
                        );
                        continue;
                    }
                    Err(e) => {
                        log::warn!("[Metadata] TMDB error: {}", e);
                        continue;
                    }
                }
            }
            "omdb" => {
                match fetch_from_omdb(client, &provider.api_key, &parsed.title, parsed.year) {
                    Ok(Some(result)) => {
                        let mut metadata = result.metadata;
                        save_metadata_and_images(
                            client,
                            app,
                            user_id,
                            media_id,
                            &mut metadata,
                            "omdb",
                            None,
                            None,
                            result.poster_url.as_deref(),
                        )?;
                        return Ok(true);
                    }
                    Ok(None) => {
                        log::info!(
                            "[Metadata] OMDb found nothing for \"{}\", trying next provider",
                            parsed.title
                        );
                        continue;
                    }
                    Err(e) => {
                        log::warn!("[Metadata] OMDb error: {}", e);
                        continue;
                    }
                }
            }
            _ => {
                log::warn!("[Metadata] Unknown provider: {}", provider.id);
                continue;
            }
        }
    }

    log::info!(
        "[Metadata] No provider could fetch metadata for \"{}\"",
        filename
    );
    Ok(false)
}

/// Fetch metadata for all entries that are missing it
pub fn fetch_missing_metadata(app: &AppHandle, user_id: &str) -> Result<String, String> {
    let settings = media::get_settings(app.clone(), user_id.to_string())?;
    let providers: Vec<media::MetadataProviderConfig> = settings
        .metadata_providers
        .into_iter()
        .filter(|p| p.enabled && !p.api_key.is_empty())
        .collect();

    if providers.is_empty() {
        log::info!("[Metadata] No enabled metadata providers configured");
        return Ok("No enabled metadata providers".to_string());
    }

    let entries = media::get_media_library(app.clone(), user_id.to_string())?;
    if entries.is_empty() {
        return Ok("No media entries to process".to_string());
    }

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let mut fetched = 0;
    let mut failed = 0;
    let mut skipped = 0;

    for entry in &entries {
        if entry.id.is_empty() {
            skipped += 1;
            continue;
        }

        if has_metadata(app, user_id, &entry.id) {
            skipped += 1;
            continue;
        }

        match fetch_metadata_for_entry(
            &client,
            app,
            user_id,
            &entry.id,
            &entry.filename,
            &providers,
        ) {
            Ok(true) => fetched += 1,
            Ok(false) => failed += 1,
            Err(e) => {
                log::warn!(
                    "[Metadata] Error fetching metadata for {}: {}",
                    entry.filename,
                    e
                );
                failed += 1;
            }
        }

        // Small delay between requests to avoid rate limiting
        std::thread::sleep(std::time::Duration::from_millis(250));
    }

    let result = format!(
        "Metadata fetch complete: {} fetched, {} failed, {} skipped",
        fetched, failed, skipped
    );
    log::info!("[Metadata] {}", result);
    Ok(result)
}

// ─── Background retry system ────────────────────────────────────────────────

/// State for the metadata retry background thread
pub struct MetadataRetryState {
    stop_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl MetadataRetryState {
    pub fn new() -> Self {
        Self {
            stop_flag: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            thread: None,
        }
    }
}

/// Start the background metadata retry loop (checks every 15 minutes)
pub fn start_metadata_retry(app: &AppHandle, user_id: &str) -> Result<(), String> {
    let state = app.state::<std::sync::Arc<std::sync::Mutex<MetadataRetryState>>>();
    let mut state = state.lock().map_err(|e| e.to_string())?;

    // Stop any existing retry thread
    state
        .stop_flag
        .store(true, std::sync::atomic::Ordering::SeqCst);
    if let Some(handle) = state.thread.take() {
        let _ = handle.join();
    }

    let stop_flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    state.stop_flag = stop_flag.clone();

    let app_clone = app.clone();
    let user_id_clone = user_id.to_string();

    let thread = std::thread::spawn(move || {
        log::info!("[Metadata] Retry thread started (interval: 15min)");

        // Wait 30 seconds before first check to let the app settle
        for _ in 0..30 {
            if stop_flag.load(std::sync::atomic::Ordering::SeqCst) {
                return;
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
        }

        loop {
            if stop_flag.load(std::sync::atomic::Ordering::SeqCst) {
                break;
            }

            match fetch_missing_metadata(&app_clone, &user_id_clone) {
                Ok(result) => log::info!("[Metadata] Retry result: {}", result),
                Err(e) => log::warn!("[Metadata] Retry failed: {}", e),
            }

            // Sleep for 15 minutes, checking stop flag every second
            for _ in 0..900 {
                if stop_flag.load(std::sync::atomic::Ordering::SeqCst) {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }

        log::info!("[Metadata] Retry thread stopped");
    });

    state.thread = Some(thread);
    Ok(())
}

/// Stop the background metadata retry loop
pub fn stop_metadata_retry(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<std::sync::Arc<std::sync::Mutex<MetadataRetryState>>>();
    let mut state = state.lock().map_err(|e| e.to_string())?;

    state
        .stop_flag
        .store(true, std::sync::atomic::Ordering::SeqCst);
    if let Some(handle) = state.thread.take() {
        let _ = handle.join();
    }

    log::info!("[Metadata] Retry thread stopped");
    Ok(())
}
