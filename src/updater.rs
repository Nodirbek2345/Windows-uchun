use self_update::backends::github::Update;
use self_update::cargo_crate_version;

pub fn update_in_background() -> Result<bool, Box<dyn std::error::Error>> {
    let status = Update::configure()
        .repo_owner("Nodirbek2345")
        .repo_name("Windows-uchun")
        .bin_name("ai-filter")
        .show_download_progress(true)
        .current_version(cargo_crate_version!())
        .build()?
        .update()?;
    
    Ok(status.updated())
}
