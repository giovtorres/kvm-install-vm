use crate::vm::DistroInfo;
use anyhow::{Context, Result};
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tracing::{debug, info, instrument, warn};

pub struct ImageManager {
    image_dir: PathBuf,
}

impl ImageManager {
    /// Create a new ImageManager with the specified image directory
    pub fn new<P: AsRef<Path>>(image_dir: P) -> Self {
        ImageManager {
            image_dir: image_dir.as_ref().to_path_buf(),
        }
    }

    /// Check if a cloud image exists locally
    pub fn image_exists(&self, distro_info: &DistroInfo) -> bool {
        let image_path = self.image_dir.join(&distro_info.qcow_filename);
        image_path.exists()
    }

    /// Get the full path to a cloud image (whether it exists or not)
    pub fn get_image_path(&self, distro_info: &DistroInfo) -> PathBuf {
        self.image_dir.join(&distro_info.qcow_filename)
    }

    /// Download a cloud image if it doesn't already exist locally
    pub async fn ensure_image(&self, distro_info: &DistroInfo) -> Result<PathBuf> {
        let image_path = self.get_image_path(distro_info);

        if image_path.exists() {
            info!("Cloud image already exists: {}", image_path.display());
            println!("Cloud image already exists: {}", image_path.display());
            return Ok(image_path);
        }

        // Create image directory if it doesn't exist
        if !self.image_dir.exists() {
            fs::create_dir_all(&self.image_dir).context("Failed to create image directory")?;
        }

        info!("Downloading cloud image: {}", distro_info.qcow_filename);
        println!("Downloading cloud image: {}", distro_info.qcow_filename);

        // Construct download URL
        let url = format!(
            "{}/{}",
            distro_info.image_url.trim_end_matches('/'),
            distro_info.qcow_filename
        );

        debug!("From URL: {}", url);
        println!("From URL: {}", url);

        // Download the file with progress indication
        self.download_file(&url, &image_path)
            .await
            .context("Failed to download cloud image")?;

        Ok(image_path)
    }

    /// Download a file with progress indication
    async fn download_file(&self, url: &str, dest: &Path) -> Result<PathBuf> {
        // Create a temporary file for downloading
        let temp_path = dest.with_extension("part");

        // Create parent directory if needed
        if let Some(parent) = temp_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Begin the download
        let res = reqwest::get(url).await?;
        let total_size = res.content_length().unwrap_or(0);

        // Setup progress bar
        let pb = ProgressBar::new(total_size);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .unwrap()
            .progress_chars("#>-"));

        // Download the file in chunks, writing each chunk to disk
        let mut file = File::create(&temp_path).await?;
        let mut downloaded: u64 = 0;
        let mut stream = res.bytes_stream();

        while let Some(item) = stream.next().await {
            let chunk = item?;
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;
            pb.set_position(downloaded);
        }

        // Ensure everything is written to disk
        file.flush().await?;

        // Finalize the download by renaming the temp file
        tokio::fs::rename(&temp_path, &dest).await?;

        pb.finish_with_message(format!("Downloaded {}", dest.display()));

        Ok(dest.to_path_buf())
    }

    /// Download a cloud image with resume capability
    #[instrument(skip(self), fields(distro = %distro_info.qcow_filename))]
    pub async fn download_image_with_resume(&self, distro_info: &DistroInfo) -> Result<PathBuf> {
        let image_path = self.image_dir.join(&distro_info.qcow_filename);
        let part_path = image_path.with_extension("part");

        // Create image directory if it doesn't exist
        if !self.image_dir.exists() {
            fs::create_dir_all(&self.image_dir).context("Failed to create image directory")?;
        }

        // Check if the image already exists
        if image_path.exists() {
            info!("Cloud image already exists: {}", image_path.display());
            println!("Cloud image already exists: {}", image_path.display());
            return Ok(image_path);
        }

        // Construct download URL
        let url = format!(
            "{}/{}",
            distro_info.image_url.trim_end_matches('/'),
            distro_info.qcow_filename
        );

        info!("Downloading cloud image: {}", distro_info.qcow_filename);
        println!("Downloading cloud image: {}", distro_info.qcow_filename);
        debug!("From URL: {}", url);

        // Check if partial download exists
        let resume_download = part_path.exists();
        if resume_download {
            info!("Partial download found. Resuming from previous download");
            println!("Partial download found. Resuming from previous download");

            let client = reqwest::Client::new();
            let file_size = part_path.metadata()?.len();

            debug!("Resuming from byte position: {}", file_size);

            // Create a request with Range header
            let mut req = client.get(&url);
            req = req.header("Range", format!("bytes={}-", file_size));

            // Download the rest of the file
            let res = req.send().await?;

            // Check if the server supports resume
            if res.status() == reqwest::StatusCode::PARTIAL_CONTENT {
                let total_size = match res.content_length() {
                    Some(len) => file_size + len,
                    None => file_size, // Just show the current size if total is unknown
                };

                // Setup progress bar
                let pb = ProgressBar::new(total_size);
                pb.set_style(ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                    .unwrap()
                    .progress_chars("#>-"));
                pb.set_position(file_size);

                // Open the existing part file for appending
                let mut file = tokio::fs::OpenOptions::new()
                    .append(true)
                    .open(&part_path)
                    .await?;

                let mut downloaded = file_size;
                let mut stream = res.bytes_stream();

                while let Some(item) = stream.next().await {
                    let chunk = item?;
                    file.write_all(&chunk).await?;
                    downloaded += chunk.len() as u64;
                    pb.set_position(downloaded);
                }

                // Ensure everything is written to disk
                file.flush().await?;

                // Finalize the download by renaming the temp file
                tokio::fs::rename(&part_path, &image_path).await?;

                pb.finish_with_message(format!("Downloaded {}", image_path.display()));

                return Ok(image_path);
            } else {
                warn!("Server does not support resume. Starting a new download");
                println!("Server does not support resume. Starting a new download");
            }
        }

        // If we got here, we need to do a full download
        self.download_file(&url, &image_path).await?;

        Ok(image_path)
    }

    /// Create a resized version of a cloud image
    pub async fn create_resized_image(
        &self,
        source_path: &Path,
        target_path: &Path,
        size_gb: u32,
    ) -> Result<()> {
        info!(
            "Creating resized image: {} ({}GB)",
            target_path.display(),
            size_gb
        );

        // Create parent directory if needed
        if let Some(parent) = target_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        // First, create a copy of the source image
        let mut cmd = Command::new("qemu-img");
        cmd.args(&[
            "create",
            "-f",
            "qcow2",
            "-F",
            "qcow2",
            "-b",
            source_path.to_str().unwrap(),
            target_path.to_str().unwrap(),
        ]);

        debug!("Executing command: {:?}", cmd);
        let status = cmd
            .status()
            .context("Failed to execute qemu-img create command")?;

        if !status.success() {
            return Err(anyhow::anyhow!("Failed to create disk image copy"));
        }

        // Then resize it to the desired size
        let mut resize_cmd = Command::new("qemu-img");
        resize_cmd.args(&[
            "resize",
            target_path.to_str().unwrap(),
            &format!("{}G", size_gb),
        ]);

        debug!("Executing command: {:?}", resize_cmd);
        let resize_status = resize_cmd
            .status()
            .context("Failed to execute qemu-img resize command")?;

        if !resize_status.success() {
            return Err(anyhow::anyhow!("Failed to resize disk image"));
        }

        info!("Successfully created and resized disk image");
        Ok(())
    }

    /// Verify the integrity of a downloaded image
    pub fn verify_image(&self, distro_info: &DistroInfo) -> Result<bool> {
        let image_path = self.get_image_path(distro_info);

        if !image_path.exists() {
            return Ok(false);
        }

        info!("Verifying image integrity: {}", image_path.display());

        // Use qemu-img check to verify the image
        let mut cmd = Command::new("qemu-img");
        cmd.args(&["check", image_path.to_str().unwrap()]);

        debug!("Executing command: {:?}", cmd);
        let output = cmd
            .output()
            .context("Failed to execute qemu-img check command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("Image verification failed: {}", stderr);
            return Ok(false);
        }

        info!("Image verification successful");
        Ok(true)
    }

    /// Delete an image from the image directory
    pub fn delete_image(&self, distro_info: &DistroInfo) -> Result<()> {
        let image_path = self.get_image_path(distro_info);

        if image_path.exists() {
            info!("Deleting image: {}", image_path.display());
            fs::remove_file(&image_path).context("Failed to delete image file")?;
            info!("Image deleted successfully");
        } else {
            info!("Image does not exist, nothing to delete");
        }

        Ok(())
    }

    /// List all available images in the image directory
    pub fn list_images(&self) -> Result<Vec<PathBuf>> {
        if !self.image_dir.exists() {
            return Ok(Vec::new());
        }

        let mut images = Vec::new();

        for entry in fs::read_dir(&self.image_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().unwrap_or_default() == "qcow2" {
                images.push(path);
            }
        }

        Ok(images)
    }
}
