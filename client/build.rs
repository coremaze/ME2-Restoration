use std::env;
use std::io::Read;
use std::io::Seek;
use std::io::Write;
use std::path::Path;
use std::process::Command;

fn get_available_targets() -> Vec<String> {
    let output = Command::new("rustc")
        .args(&["--print", "target-list"])
        .output()
        .expect("Failed to run rustc");

    String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(String::from)
        .collect()
}

fn get_installed_targets() -> Vec<String> {
    let output = Command::new("rustup")
        .args(&["target", "list"])
        .output()
        .expect("Failed to run rustup");

    String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .filter(|line| line.contains("installed"))
        .map(|line| line.split_whitespace().next().unwrap().to_string())
        .collect()
}

fn target_help_string(profile: &str) -> String {
    let available_targets = get_available_targets();
    let installed_targets = get_installed_targets();
    let available_windows_i686 = available_targets
        .iter()
        .filter(|t| t.starts_with("i686-pc-windows"))
        .collect::<Vec<&String>>();
    let installed_windows_i686 = installed_targets
        .iter()
        .filter(|t| t.starts_with("i686-pc-windows"))
        .collect::<Vec<&String>>();

    let mut lines = vec![
        "".to_string(),
        "Target must be 32 bit Windows.".to_string(),
        "".to_string(),
    ];

    if available_windows_i686.len() > 0 {
        lines.push("Available targets:".to_string());
        for target in &available_windows_i686 {
            lines.push(format!("  {}", target));
        }
    } else {
        lines.push("No available 32 bit Windows targets.".to_string());
    }
    lines.push("".to_string());

    if installed_windows_i686.len() > 0 {
        lines.push("Installed targets:".to_string());
        for target in &installed_windows_i686 {
            lines.push(format!("  {}", target));
        }
    } else {
        lines.push(format!(
            "You have no installed 32 bit Windows targets. Install a target with `rustup target add <target>`."
        ));
    }
    lines.push("".to_string());

    if installed_windows_i686.len() > 0 {
        lines.push("Run any of the following commands to build:".to_string());
        for target in &installed_windows_i686 {
            let profile_arg = if profile == "release" {
                "--release "
            } else {
                ""
            };
            lines.push(format!("  cargo build {profile_arg}--target {target}"));
        }
    }
    lines.push("".to_string());

    panic!("{}", lines.join("\n"));
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    std::fs::create_dir_all(&dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn zip_dir<T>(
    it: &mut dyn Iterator<Item = walkdir::DirEntry>,
    prefix: &Path,
    writer: T,
    method: zip::CompressionMethod,
) where
    T: Write + Seek,
{
    let mut zip = zip::ZipWriter::new(writer);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(method)
        .unix_permissions(0o755);

    let prefix = Path::new(prefix);
    let mut buffer = Vec::new();
    for entry in it {
        let path = entry.path();
        let name = path.strip_prefix(prefix).unwrap();
        let path_as_string = name
            .to_str()
            .map(str::to_owned)
            .expect("{name:?} Is a Non UTF-8 Path");

        // Write file or directory explicitly
        // Some unzip tools unzip files with directory paths correctly, some do not!
        if path.is_file() {
            // println!("adding file {path:?} as {name:?} ...");
            zip.start_file(path_as_string, options).unwrap();
            let mut f = std::fs::File::open(path).unwrap();

            f.read_to_end(&mut buffer).unwrap();
            zip.write_all(&buffer).unwrap();
            buffer.clear();
        } else if !name.as_os_str().is_empty() {
            // Only if not root! Avoids path spec / warning
            // and mapname conversion failed error on unzip
            // println!("adding dir {path_as_string:?} as {name:?} ...");
            zip.add_directory(path_as_string, options).unwrap();
        }
    }
    zip.finish().unwrap();
}

fn main() {
    // Get output directory from cargo
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let target_dir = Path::new(&manifest_dir).join("target");
    let package_dir = target_dir.join("me2_game");
    std::fs::create_dir_all(&package_dir).expect("Failed to create output directory");
    let zip_path = target_dir.join("me2_game.zip");

    // Get target and mode
    let target = env::var("CARGO_BUILD_TARGET")
        .or_else(|_| env::var("TARGET"))
        .expect("Could not determine target");
    let profile = env::var("PROFILE").expect("Could not determine build profile");

    // Make sure the target is windows i686
    if !target.starts_with("i686-pc-windows") {
        target_help_string(&profile);
    }

    // List of member projects to build
    let projects = vec![("me2hook", "dll"), ("me2_launcher", "exe")];

    for (project, ext) in projects {
        // Build each project
        let project_path = Path::new(&manifest_dir).join(project);

        println!("cargo:rerun-if-changed={}", project_path.display());

        let profile_arg = if profile == "release" {
            "--release"
        } else {
            ""
        };

        let status = Command::new("cargo")
            .current_dir(&project_path)
            .args(&["build", "--target", &target, &profile_arg])
            .status()
            .expect(&format!("Failed to build {}", project));

        if !status.success() {
            panic!("Failed to build {}", project);
        }

        // Copy the built artifacts to the output directory
        let target_dir =
            Path::new(&manifest_dir).join(format!("{}/target/{}/{}", project, target, profile));
        let binary_name = project; // Assuming binary name matches project name

        let binary_path = target_dir.join(format!("{}.{}", binary_name, ext));
        let dest_path = Path::new(&package_dir).join(format!("{}.{}", binary_name, ext));

        std::fs::copy(&binary_path, &dest_path).expect(&format!(
            "Failed to copy {:?} to output directory {:?}",
            binary_path, dest_path
        ));
    }

    // Copy game/me2/ and game/projector/ to the output directory
    let game_path = Path::new(&manifest_dir).join("game");
    let me2_path = game_path.join("me2");
    let projector_path = game_path.join("projector");

    let me2_dest_path = Path::new(&package_dir).join("me2");
    let projector_dest_path = Path::new(&package_dir).join("projector");

    copy_dir_all(&me2_path, &me2_dest_path).expect(&format!(
        "Failed to copy {:?} to output directory {:?}",
        me2_path, me2_dest_path
    ));
    copy_dir_all(&projector_path, &projector_dest_path).expect(&format!(
        "Failed to copy {:?} to output directory {:?}",
        projector_path, projector_dest_path
    ));

    // add all the files to a zip
    let file = std::fs::File::create(&zip_path).unwrap();
    let walkdir = walkdir::WalkDir::new(&package_dir);
    zip_dir(
        &mut walkdir.into_iter().filter_map(|e| e.ok()),
        &package_dir,
        file,
        zip::CompressionMethod::Deflated,
    );

    // Print cargo instructions
    println!("cargo:rerun-if-changed=build.rs");
}
