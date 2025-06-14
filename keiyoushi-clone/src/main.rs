use std::{ path::Path, str::FromStr};

use reqwest::Url;
use serde::{Deserialize, Serialize};
use tokio::{
    fs::File,
    io::{AsyncWriteExt, BufWriter},
};

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
const WANTED_LANGS: [&str; 2] = ["pt-BR", "en"];
const KEIYOUSHI_SOURCE_URL: &str =
    "https://raw.githubusercontent.com/keiyoushi/extensions/refs/heads/repo/index.json";
const KEIYOUSHI_SOURCE_BIN_URL: &str =
    "https://raw.githubusercontent.com/keiyoushi/extensions/refs/heads/repo/apk/";
const KEIYOUSHI_SOURCE_ICON_URL: &str =
    "https://raw.githubusercontent.com/keiyoushi/extensions/refs/heads/repo/icon/";

async fn download_icon(bin_path: &Url) -> Result<(), String> {
    let file_name = bin_path
        .path_segments()
        .and_then(|segments| segments.last())
        .ok_or("Invalid URL: no file name found")?;

    let file_path = Path::new("icon").join(file_name);

    let response = reqwest::get(bin_path.to_owned())
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!("Failed to download file: {}", response.status()).into());
    }
    let mut file = BufWriter::new(File::create(&file_path).await.map_err(|e| e.to_string())?);

    let mut stream = response.bytes_stream();

    use futures_util::StreamExt;
    while let Some(chunk) = stream.next().await {
        let data = chunk.map_err(|e| e.to_string())?;
        file.write_all(&data).await.map_err(|e| e.to_string())?;
    }

    file.flush().await.map_err(|e| e.to_string())?;
    Ok(())
}

async fn download_bin(bin_path: &Url) -> Result<(), String> {
    let file_name = bin_path
        .path_segments()
        .and_then(|segments| segments.last())
        .ok_or("Invalid URL: no file name found")?;

    let file_path = Path::new("apk").join(file_name);

    let response = reqwest::get(bin_path.to_owned())
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!("Failed to download file: {}", response.status()).into());
    }
    let mut file = BufWriter::new(File::create(&file_path).await.map_err(|e| e.to_string())?);

    let mut stream = response.bytes_stream();

    use futures_util::StreamExt;
    while let Some(chunk) = stream.next().await {
        let data = chunk.map_err(|e| e.to_string())?;
        file.write_all(&data).await.map_err(|e| e.to_string())?;
    }

    file.flush().await.map_err(|e| e.to_string())?;
    Ok(())
}

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

    let keiyoushi_bin_base_url =
        Url::from_str(KEIYOUSHI_SOURCE_BIN_URL).map_err(|e| e.to_string())?;

    let keiyoushi_icon_base_url =
        Url::from_str(KEIYOUSHI_SOURCE_ICON_URL).map_err(|e| e.to_string())?;

    let apk_bins: Vec<Url> = packages
        .iter()
        .map(|package| keiyoushi_bin_base_url.join(&package.apk).unwrap())
        .collect();

    let source_icons: Vec<Url> = packages
        .iter()
        .map(|package| {
            keiyoushi_icon_base_url
                .join(&format!("{}{}", &package.pkg, ".png"))
                .unwrap()
        })
        .collect();

    let apk_bin_futures = apk_bins.iter().map(|url| download_bin(url));
    let icon_futures = source_icons.iter().map(|url| download_icon(url));


    futures::future::join_all(apk_bin_futures).await;
    futures::future::join_all(icon_futures).await;

    let packages_str = serde_json::to_string(&packages).map_err(|e| e.to_string())?;

    tokio::fs::write(INDEX_FILE_PATH.to_owned(), packages_str)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}
