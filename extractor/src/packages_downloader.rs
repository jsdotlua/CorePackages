//! Fetches the latest Roblox Studio deployment for Windows and downloads only the LuaPackages directory.
//!
//! Thanks to the way Roblox deploys Windows clients, we can speed things up here by only downloading the directory we
//! need, rather than the entire Studio release.

use std::{io::Cursor, path::Path, str::FromStr};

use anyhow::{bail, Context};
use futures::{future, StreamExt};
use reqwest::{
    header::{HeaderValue, CONTENT_LENGTH, RANGE},
    Client, StatusCode,
};
use roblox_version_archive::prelude::{
    get_latest_deployment, BinaryType, DeploymentSpace, PrimaryChannel,
};

use crate::zip_extract::extract_zip;

/// Path at Roblox's CDN that contains our LuaPackages
const DOWNLOAD_FILE: &str = "extracontent-luapackages.zip";

/// Target parallel jobs for downloading LuaPackages. Too many will make downloads slower.
const TARGET_DOWNLOAD_JOBS: u32 = 2;

const DEPLOYMENT_SPACE: DeploymentSpace = DeploymentSpace::Global;
const BINARY_TYPE: BinaryType = BinaryType::WindowsStudio64;
const CHANNEL: PrimaryChannel = PrimaryChannel::Live;

/// Downloads the latest LuaPackages and extracts it to the given Path.
pub async fn download_latest_lua_packages(extract_to: &Path) -> anyhow::Result<()> {
    let client = Client::new();

    log::info!("Fetching latest Studio release");

    let latest_release = get_latest_deployment(&DEPLOYMENT_SPACE, &BINARY_TYPE, &CHANNEL, &client)
        .await
        .context("Failed to get latest deployment")?;

    let cdn_path = format!("https://setup.{}", DEPLOYMENT_SPACE.get_cdn_domain());
    let download_path = format!(
        "{cdn_path}/{}-{DOWNLOAD_FILE}",
        latest_release.client_version
    );

    log::info!("Downloading LuaPackages from {download_path}");

    let file_bytes = download_file(&client, &download_path, TARGET_DOWNLOAD_JOBS)
        .await
        .context("Failed to download LuaPackages directory from CDN")?;

    log::info!("Extracting LuaPackages to {extract_to:?}");

    extract_zip(Cursor::new(&file_bytes), extract_to, false)
        .context(format!("Failed to extract LuaPackages to {extract_to:?}"))?;

    Ok(())
}

/// Download a file from an AWS CDN using the `RANGE` header for faster download.
async fn download_file(
    client: &Client,
    url: &str,
    target_download_jobs: u32,
) -> anyhow::Result<Vec<u8>> {
    log::debug!("Starting download of {url}");

    // Get the content length so we can download the file in parallel chunks
    let response = client
        .head(url)
        .send()
        .await
        .context(format!("Failed to make HEAD reqwest to {url}"))?;

    let content_length = response
        .headers()
        .get(CONTENT_LENGTH)
        .context("HEAD response does not include content length")?
        .to_str()
        .context("Failed to convert content length to string slice")?;

    let content_length =
        u64::from_str(content_length).context("Failed to convert string slice to u64")?;

    log::debug!("Content length for {url}: {content_length}");

    // Start downloading chunks
    log::debug!("Downloading file at {url}");

    let buffer_size = content_length.div_floor(target_download_jobs as u64);
    let range_iter = PartialRangeIter::new(0, content_length - 1, buffer_size)
        .context("Failed to make range iter")?;

    log::debug!("Download chunks for {url}: {}", range_iter.clone().count());

    // Make a list of all async download jobs and await them all together
    let mut download_tasks = Vec::new();
    for range in range_iter {
        let task = download_partial_chunk(client, url, range);
        download_tasks.push(task);
    }

    let downloaded_chunks = future::try_join_all(download_tasks)
        .await
        .context("Failed to download {url}")?;

    log::debug!("Downloaded file at {url}");

    // Join all downloaded chunks into one byte array and write to path
    let mut file_bytes = Vec::new();
    for mut chunk in downloaded_chunks {
        file_bytes.append(&mut chunk);
    }

    log::debug!("Completed download of {url}");

    Ok(file_bytes)
}

/// Download a partial file chunk from the CDN in parallel to speed up download
async fn download_partial_chunk(
    client: &Client,
    url: &str,
    range: HeaderValue,
) -> anyhow::Result<Vec<u8>> {
    log::trace!("Range {range:?} ({url})");

    let response = client
        .get(url)
        .header(RANGE, &range)
        .send()
        .await
        .context("Request for range {range:?} at {url} failed")?;

    let status = response.status();
    if !(status == StatusCode::OK || status == StatusCode::PARTIAL_CONTENT) {
        bail!("Got unexpected response from CDN ({url} {range:?}): {status}");
    }

    let mut stream = response.bytes_stream();
    let mut bytes = Vec::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;

        for byte in chunk.into_iter() {
            bytes.push(byte);
        }
    }

    Ok(bytes)
}

/// https://rust-lang-nursery.github.io/rust-cookbook/web/clients/download.html#make-a-partial-download-with-http-range-headers
#[derive(Debug, Clone)]
struct PartialRangeIter {
    start: u64,
    end: u64,
    buffer_size: u64,
}

impl PartialRangeIter {
    pub fn new(start: u64, end: u64, buffer_size: u64) -> anyhow::Result<Self> {
        if buffer_size == 0 {
            bail!("Expected a value greater than 0 for buffer_size, got {buffer_size}.");
        }

        Ok(PartialRangeIter {
            start,
            end,
            buffer_size,
        })
    }
}

impl Iterator for PartialRangeIter {
    type Item = HeaderValue;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start > self.end {
            None
        } else {
            let prev_start = self.start;
            self.start += std::cmp::min(self.buffer_size, self.end - self.start + 1);

            let header = HeaderValue::from_str(&format!("bytes={}-{}", prev_start, self.start - 1))
                .expect("string provided by format!");

            Some(header)
        }
    }
}
