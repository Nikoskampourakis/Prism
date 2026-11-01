use directories::ProjectDirs;
use slint::ComponentHandle;
use std::path::PathBuf;

slint::include_modules!();

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = MainWindow::new()?;
    let catalog_path = ProjectDirs::from("org", "Prism", "Prism")
        .map(|directories| directories.data_dir().join("vms.json"))
        .unwrap_or_else(|| PathBuf::from("prism-vms.json"));
    let vm_count = prism::storage::VmCatalog::new(catalog_path).load()?.len();
    app.set_vm_count(vm_count as i32);
    app.set_profile_xml(include_str!("../examples/basic-android.xml").into());

    let weak_app = app.as_weak();
    app.on_preview_config(move |source| {
        let result = match prism::config::parse_and_validate(source.as_str()) {
            Ok(preview) => {
                let operations = preview.operations.iter().map(|operation| format!("- {operation:?}")).collect::<Vec<_>>().join("\n");
                let unsupported = if preview.unsupported.is_empty() {
                    "None".to_string()
                } else {
                    preview.unsupported.iter().map(|item| format!("- {item}")).collect::<Vec<_>>().join("\n")
                };
                format!(
                    "VALID PROFILE · {} · Android {}\n{} MiB RAM · {} GiB storage · confirmation required: {}\nPlanned operations:\n{}\nUnavailable here:\n{}",
                    preview.name,
                    preview.android_version,
                    preview.ram_mib,
                    preview.storage_gib,
                    if preview.requires_confirmation { "yes" } else { "no" },
                    if operations.is_empty() { "- No operations" } else { &operations },
                    unsupported,
                )
            }
            Err(error) => format!("INVALID PROFILE\n{error}"),
        };
        if let Some(app) = weak_app.upgrade() {
            app.set_profile_result(result.into());
        }
    });

    app.run()?;
    Ok(())
}
