use std::path::Path;
use std::process::Command;

pub fn run_python_transcriber(
    input_path: &Path,
    transcript_json_path: &Path,
    transcript_txt_path: &Path,
    analysis_path: Option<&Path>,
) -> Result<(), String> {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let project_root = manifest_dir
        .parent()
        .ok_or_else(|| "failed to resolve project root".to_string())?;
    let script = project_root.join("python").join("transcriber").join("main.py");

    if !script.exists() {
        return Err(format!("transcriber script not found at {}", script.display()));
    }

    let mut command = Command::new("python3");
    command
        .arg(script)
        .arg("--input")
        .arg(input_path)
        .arg("--output-json")
        .arg(transcript_json_path)
        .arg("--output-txt")
        .arg(transcript_txt_path);

    if let Some(path) = analysis_path {
        command.arg("--analysis").arg(path);
    }

    let output = command
        .output()
        .map_err(|e| format!("failed to run python transcriber: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(format!(
            "transcriber failed with status {}\nstdout:\n{}\nstderr:\n{}",
            output.status, stdout, stderr
        ));
    }

    Ok(())
}
