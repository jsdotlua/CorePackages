use std::{
    fs,
    io::{self, Read, Seek},
    path::{Path, PathBuf},
};

use anyhow::Context;

// Forked from https://github.com/MCOfficer/zip-extract/blob/master/src/lib.rs
pub fn extract_zip<S: Read + Seek>(
    source: S,
    target_dir: &Path,
    strip_toplevel: bool,
) -> anyhow::Result<()> {
    if !target_dir.exists() {
        fs::create_dir_all(&target_dir)?;
    }

    let mut archive = zip::ZipArchive::new(source)?;
    let do_strip_toplevel = strip_toplevel && has_toplevel(&mut archive)?;

    log::debug!("Extracting to {}", target_dir.to_string_lossy());

    let archive_size = archive.len();

    for i in 0..archive_size {
        let mut file = archive.by_index(i)?;
        let mut relative_path = file.mangled_name();

        if do_strip_toplevel {
            let base = relative_path
                .components()
                .take(1)
                .fold(PathBuf::new(), |mut p, c| {
                    p.push(c);
                    p
                });

            relative_path = relative_path
                .strip_prefix(&base)
                .context("Failed to strip prefix")?
                .to_path_buf()
        }

        if relative_path.to_string_lossy().is_empty() {
            // Top-level directory
            continue;
        }

        let mut outpath = target_dir.to_path_buf();
        outpath.push(relative_path);

        log::trace!(
            "Extracting {} to {}",
            file.name(),
            outpath.to_string_lossy()
        );

        if file.is_dir() {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(&p)?;
                }
            }

            let mut outfile = fs::File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;
        }

        #[cfg(unix)]
        set_unix_mode(&file, &outpath)?;
    }

    log::debug!("Extracted {} files", archive.len());

    Ok(())
}

fn has_toplevel<S: Read + Seek>(
    archive: &mut zip::ZipArchive<S>,
) -> Result<bool, zip::result::ZipError> {
    let mut toplevel_dir: Option<PathBuf> = None;
    if archive.len() < 2 {
        return Ok(false);
    }

    for i in 0..archive.len() {
        let file = archive.by_index(i)?.mangled_name();

        if let Some(toplevel_dir) = &toplevel_dir {
            if !file.starts_with(toplevel_dir) {
                log::trace!("Found different toplevel directory");

                return Ok(false);
            }
        } else {
            // First iteration
            let comp: PathBuf = file.components().take(1).collect();
            log::trace!(
                "Checking if path component {} is the only toplevel directory",
                comp.to_string_lossy()
            );

            toplevel_dir = Some(comp);
        }
    }

    log::trace!("Found no other toplevel directory");

    Ok(true)
}

#[cfg(unix)]
fn set_unix_mode(file: &zip::read::ZipFile, outpath: &Path) -> io::Result<()> {
    use std::os::unix::prelude::PermissionsExt;

    if let Some(m) = file.unix_mode() {
        fs::set_permissions(&outpath, PermissionsExt::from_mode(m))?
    }
    Ok(())
}
