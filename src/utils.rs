use std::process::Command;
use std::io::{BufReader, BufRead};

pub fn round_to_2_decimals(value: f32) -> f32 {
    // Use string formatting to get exactly 2 decimal places and then parse back
    // This avoids floating point precision issues
    format!("{:.2}", value).parse().unwrap_or(value)
}

/// Executes a bash command to find running processes that use Vulkan
/// and are owned by the current user.
/// Returns a vector of process descriptions with PID and name.
pub fn get_vulkan_processes() -> Vec<String> {
    let mut processes = Vec::new();
    let command_str = r#"
        for pid in /proc/[0-9]*; do
            owner=$(stat -c %U "$pid" 2>/dev/null)
            if [[ "$owner" == "$USER" ]]; then
                if grep -qi 'vulkan' "$pid/maps" 2>/dev/null; then
                    procname=$(cat "$pid/comm" 2>/dev/null)
                    if [[ -n "$procname" ]]; then
                        printf "PID %s: %s\n" "$(basename "$pid")" "$procname"
                    fi
                fi
            fi
        done
    "#;

    // Execute the bash command
    let output = Command::new("bash")
        .arg("-c")
        .arg(command_str)
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                // Read stdout line by line
                let reader = BufReader::new(output.stdout.as_slice());
                for line in reader.lines() {
                    if let Ok(proc_info) = line {
                        let trimmed_info = proc_info.trim().to_string();
                        if !trimmed_info.is_empty() {
                            processes.push(trimmed_info);
                        }
                    }
                }
            } else {
                // Print stderr if the command failed
                eprintln!("Command failed with error: {}", String::from_utf8_lossy(&output.stderr));
            }
        },
        Err(e) => {
            // Print error if the command could not be executed
            eprintln!("Failed to execute command: {}", e);
        }
    }
    processes
}

