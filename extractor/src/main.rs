use std::{env, fs};

use anyhow::{bail, Context};
use env_logger::Env;
use extractor::{
    documentation::{generate_readme, ReadmeContent},
    package_registry::{PackageRef, PackageRegistry},
};
// use extractor::packages_downloader::download_latest_lua_packages;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let env = Env::default()
        .filter_or("LOG_LEVEL", "info")
        .write_style_or("LOG_STYLE", "always");

    env_logger::init_from_env(env);

    let current_dir = env::current_dir()?;
    let temp_path = current_dir.join("temp");

    if !temp_path.exists() {
        log::info!("Missing temp dir, creating new");
        fs::create_dir_all(&temp_path).context("Failed to create temp dir")?;
    }

    let packages_dir = temp_path.join("LuaPackages");
    // if packages_dir.exists() {
    //     log::warn!("LuaPackages dir already exists. Removing.");
    //     fs::remove_dir_all(&packages_dir).context("Failed to remove LuaPackages dir")?;
    // }

    // download_latest_lua_packages(&packages_dir).await.unwrap();

    let index_path = packages_dir.join("Packages/_Index");
    if !index_path.exists() {
        bail!("Index path does not exist at: {index_path:?}");
    }

    let mut registry = PackageRegistry::new().context("Failed to create package registry")?;

    registry
        .discover_packages_at_index(&index_path)
        .context("Failed to discover packages")?;

    // println!("\n\nDependency tree for Jest:\n");

    // let (_, package) = registry.find_by_path_name("Jest-edcba0e9-3.2.1").unwrap();
    // println!("{}", package.generate_package_tree(&registry)?);

    let readme = generate_readme(
        &registry,
        &ReadmeContent {
            available_packages: vec![
                PackageRef(1),
                PackageRef(2),
                PackageRef(3),
                PackageRef(4),
                PackageRef(5),
            ],
            blocked_packages: vec![
                PackageRef(1),
                PackageRef(2),
                PackageRef(3),
                PackageRef(4),
                PackageRef(5),
            ],
            blocking_packages: vec![
                PackageRef(1),
                PackageRef(2),
                PackageRef(3),
                PackageRef(4),
                PackageRef(5),
            ],
            unlicensed_packages: vec![
                PackageRef(1),
                PackageRef(2),
                PackageRef(3),
                PackageRef(4),
                PackageRef(5),
            ],
        },
    )?;

    fs::write("TEST.md", &readme).unwrap();

    Ok(())
}
