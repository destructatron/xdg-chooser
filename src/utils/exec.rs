use std::process::Command;

use anyhow::{Context, Result};

use crate::desktop::entry::AppEntry;

/// Parsed exec command
struct ParsedExec {
    program: String,
    args: Vec<String>,
}

/// Remove desktop entry field codes from an exec string
/// Field codes: %f, %F, %u, %U, %d, %D, %n, %N, %i, %c, %k, %v, %m
fn remove_field_codes(exec: &str) -> String {
    let mut result = String::with_capacity(exec.len());
    let mut chars = exec.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '%' {
            if let Some(&next) = chars.peek() {
                // Check if this is a field code
                if matches!(next, 'f' | 'F' | 'u' | 'U' | 'd' | 'D' | 'n' | 'N' | 'i' | 'c' | 'k' | 'v' | 'm' | '%')
                {
                    if next == '%' {
                        // %% means literal %
                        result.push('%');
                    }
                    chars.next(); // consume the field code character
                    continue;
                }
            }
        }
        result.push(c);
    }

    result
}

/// Parse an Exec line from a desktop entry using proper shell-like parsing
fn parse_exec(exec: &str) -> Result<ParsedExec> {
    // First remove field codes
    let cleaned = remove_field_codes(exec);

    // Parse using shell-words to handle quotes and escapes properly
    let parts = shell_words::split(&cleaned)
        .with_context(|| format!("Invalid Exec command syntax: {}", exec))?;

    let program = parts
        .first()
        .context("Empty Exec command")?
        .to_string();

    let args = parts.into_iter().skip(1).collect();

    Ok(ParsedExec { program, args })
}

/// Launch an application for testing
pub fn launch_app(app: &AppEntry) -> Result<()> {
    let exec = app.exec.as_ref().context("No Exec field in desktop entry")?;

    let parsed = parse_exec(exec)?;

    let mut cmd = Command::new(&parsed.program);
    cmd.args(&parsed.args);

    // Detach from parent process using process_group (safer than pre_exec + setsid)
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        // Create a new process group so the child doesn't receive signals from this terminal
        cmd.process_group(0);
    }

    cmd.spawn()
        .with_context(|| format!("Failed to launch {}", parsed.program))?;

    Ok(())
}

/// Launch an application with a file argument
pub fn launch_app_with_file(app: &AppEntry, file_path: &str) -> Result<()> {
    let exec = app.exec.as_ref().context("No Exec field in desktop entry")?;

    // Check if the exec line contains file-related field codes
    let has_file_codes = exec.contains("%f")
        || exec.contains("%F")
        || exec.contains("%u")
        || exec.contains("%U");

    // Replace file field codes with the file path
    let exec_with_file = exec
        .replace("%f", file_path)
        .replace("%F", file_path)
        .replace("%u", file_path)
        .replace("%U", file_path);

    let parsed = parse_exec(&exec_with_file)?;

    let mut cmd = Command::new(&parsed.program);
    cmd.args(&parsed.args);

    // Only add file as explicit argument if no field codes were present
    if !has_file_codes {
        cmd.arg(file_path);
    }

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }

    cmd.spawn()
        .with_context(|| format!("Failed to launch {} with {}", parsed.program, file_path))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_field_codes() {
        assert_eq!(remove_field_codes("firefox %u"), "firefox ");
        assert_eq!(remove_field_codes("gedit %F"), "gedit ");
        assert_eq!(remove_field_codes("app --flag %f --other"), "app --flag  --other");
        assert_eq!(remove_field_codes("echo %%"), "echo %");
        assert_eq!(remove_field_codes("no codes here"), "no codes here");
    }

    #[test]
    fn test_parse_exec_with_quotes() {
        let parsed = parse_exec(r#""/usr/bin/my app" --flag "value with spaces""#).unwrap();
        assert_eq!(parsed.program, "/usr/bin/my app");
        assert_eq!(parsed.args, vec!["--flag", "value with spaces"]);
    }

    #[test]
    fn test_parse_exec_simple() {
        let parsed = parse_exec("firefox --new-window").unwrap();
        assert_eq!(parsed.program, "firefox");
        assert_eq!(parsed.args, vec!["--new-window"]);
    }
}
