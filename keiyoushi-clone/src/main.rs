use std::str::FromStr;

use reqwest::Url;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TachiyomiPkg {
    pub name: String,
    pub pkg: String,
    pub apk: String,
    pub lang: String,
    pub code: i64,
    pub version: String,
    pub nsfw: i64,
    pub sources: Vec<Source>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    pub name: String,
    pub lang: String,
    pub id: String,
    pub base_url: String,
}

const INDEX_FILE_PATH: &str = "index.json";

const WANTED_PACKAGES: [&str; 1] = ["MangaDex"];
const WANTED_LANGS: [&str; 1] = ["pt-BR"];
const KEIYOUSHI_SOURCE_URL: &str =
    "https://raw.githubusercontent.com/keiyoushi/extensions/refs/heads/repo/index.json";

#[tokio::main]
async fn main() -> Result<(), String> {
    let keiyoushi_source: Url = Url::from_str(KEIYOUSHI_SOURCE_URL).map_err(|e| e.to_string())?;

    let keiyoushi_json_response = reqwest::get(keiyoushi_source)
        .await
        .map_err(|e| e.to_string())?;

    if !keiyoushi_json_response.status().is_success() {
        let err_msg = format!(
            "Request to keiyoushi source failed with status {}",
            keiyoushi_json_response.status()
        );
        return Err(err_msg);
    }

    let keiyoushi_json_text_content = keiyoushi_json_response
        .text()
        .await
        .map_err(|e| e.to_string())?;

    let keiyoushi_json = serde_json::from_str::<Vec<TachiyomiPkg>>(&keiyoushi_json_text_content)
        .map_err(|e| e.to_string())?;

    let packages: Vec<TachiyomiPkg> = keiyoushi_json
        .iter()
        .filter(|pkg| WANTED_PACKAGES.iter().any(|name| pkg.name.contains(name)))
        .map(|pkg| {
            let mut pkg = pkg.clone();
            pkg.sources.retain(|source| {
                WANTED_LANGS.iter().any(|lang| lang == &source.lang)
                    && WANTED_PACKAGES.iter().any(|name| name == &source.name)
            });
            pkg
        })
        .collect();

    let packages_str = serde_json::to_string(&packages).map_err(|e| e.to_string())?;

    tokio::fs::write(INDEX_FILE_PATH.to_owned(), packages_str).await.map_err(|e| e.to_string())?;

    Ok(())
}
