use std::path::Path;
use std::process::Command;

pub fn run_python_analyzer(input_path: &Path, output_path: &Path) -> Result<(), String> {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let project_root = manifest_dir
        .parent()
        .ok_or_else(|| "failed to resolve project root".to_string())?;
    let script = project_root.join("python").join("analyzer").join("main.py");

    if !script.exists() {
        return Err(format!("analyzer script not found at {}", script.display()));
    }

    let output = Command::new("python3")
        .arg(script)
        .arg("--input")
        .arg(input_path)
        .arg("--output")
        .arg(output_path)
        .output()
        .map_err(|e| format!("failed to run python analyzer: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(format!(
            "analyzer failed with status {}\nstdout:\n{}\nstderr:\n{}",
            output.status, stdout, stderr
        ));
    }

    Ok(())
}
