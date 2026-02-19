use std::io::Write;

use anyhow::Result;

use crate::db::model::EnvironmentRow;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RawOutputFormat {
    Kv,
    Sh,
    Fish,
    Nu,
    Cmd,
    PowerShell,
}

fn quote_sh_value(value: &str) -> String {
    let mut quoted = String::with_capacity(value.len() + 2);
    quoted.push('\'');

    for ch in value.chars() {
        if ch == '\'' {
            quoted.push_str("'\"'\"'");
        } else {
            quoted.push(ch);
        }
    }

    quoted.push('\'');
    quoted
}

fn quote_fish_value(value: &str) -> String {
    let mut quoted = String::with_capacity(value.len() + 2);
    quoted.push('\'');

    for ch in value.chars() {
        match ch {
            '\\' => quoted.push_str("\\\\"),
            '\'' => quoted.push_str("\\'"),
            _ => quoted.push(ch),
        }
    }

    quoted.push('\'');
    quoted
}

fn escape_cmd_value(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '^' => escaped.push_str("^^"),
            '"' => escaped.push_str("^\""),
            _ => escaped.push(ch),
        }
    }

    escaped
}

fn escape_nu_value(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            _ => escaped.push(ch),
        }
    }

    escaped
}

fn escape_powershell_value(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '`' => escaped.push_str("``"),
            '"' => escaped.push_str("`\""),
            '$' => escaped.push_str("`$"),
            _ => escaped.push(ch),
        }
    }

    escaped
}

pub fn write_nu_record<W: Write>(writer: &mut W, envs: &[EnvironmentRow]) -> Result<()> {
    write!(writer, "{{")?;

    for (idx, env) in envs.iter().enumerate() {
        if idx > 0 {
            write!(writer, ", ")?;
        }

        write!(
            writer,
            "\"{}\": \"{}\"",
            escape_nu_value(&env.key),
            escape_nu_value(&env.value)
        )?;
    }

    writeln!(writer, "}}")?;

    Ok(())
}

pub fn write_raw_entry<W: Write>(
    writer: &mut W,
    key: &str,
    value: &str,
    output_format: RawOutputFormat,
) -> Result<()> {
    match output_format {
        RawOutputFormat::Kv => writeln!(writer, "{}={}", key, value)?,
        RawOutputFormat::Sh => writeln!(writer, "export {}={}", key, quote_sh_value(value))?,
        RawOutputFormat::Fish => writeln!(writer, "set -gx {} {}", key, quote_fish_value(value))?,
        RawOutputFormat::Nu => unreachable!("nu output is rendered as a single record"),
        RawOutputFormat::Cmd => writeln!(writer, "set \"{}={}\"", key, escape_cmd_value(value))?,
        RawOutputFormat::PowerShell => writeln!(
            writer,
            "$env:{} = \"{}\"",
            key,
            escape_powershell_value(value)
        )?,
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render_entry(output_format: RawOutputFormat, key: &str, value: &str) -> String {
        let mut output = Vec::new();
        write_raw_entry(&mut output, key, value, output_format).unwrap();
        String::from_utf8(output).unwrap()
    }

    fn render_nu_record(envs: &[EnvironmentRow]) -> String {
        let mut output = Vec::new();
        write_nu_record(&mut output, envs).unwrap();
        String::from_utf8(output).unwrap()
    }

    #[test]
    fn test_write_raw_entry_kv_output_cases() {
        let cases = [
            ("KEY", "plain", "KEY=plain\n"),
            (
                "KEY",
                "value with spaces and 'quote' and $HOME",
                "KEY=value with spaces and 'quote' and $HOME\n",
            ),
            ("KEY", "", "KEY=\n"),
        ];

        for (key, value, expected) in cases {
            assert_eq!(expected, render_entry(RawOutputFormat::Kv, key, value));
        }
    }

    #[test]
    fn test_write_raw_entry_sh_output_cases() {
        let cases = [
            ("KEY", "plain", "export KEY='plain'\n"),
            (
                "KEY",
                "value with 'single' and $HOME",
                "export KEY='value with '\"'\"'single'\"'\"' and $HOME'\n",
            ),
            ("KEY", "", "export KEY=''\n"),
        ];

        for (key, value, expected) in cases {
            assert_eq!(expected, render_entry(RawOutputFormat::Sh, key, value));
        }
    }

    #[test]
    fn test_write_raw_entry_fish_output_cases() {
        let cases = [
            ("KEY", "plain", "set -gx KEY 'plain'\n"),
            (
                "KEY",
                "path\\to\\it's",
                "set -gx KEY 'path\\\\to\\\\it\\'s'\n",
            ),
            ("KEY", "", "set -gx KEY ''\n"),
        ];

        for (key, value, expected) in cases {
            assert_eq!(expected, render_entry(RawOutputFormat::Fish, key, value));
        }
    }

    #[test]
    fn test_write_raw_entry_cmd_output_cases() {
        let cases = [
            ("KEY", "plain", "set \"KEY=plain\"\n"),
            ("KEY", "a^b\"c%PATH%d", "set \"KEY=a^^b^\"c%PATH%d\"\n"),
            ("KEY", "", "set \"KEY=\"\n"),
        ];

        for (key, value, expected) in cases {
            assert_eq!(expected, render_entry(RawOutputFormat::Cmd, key, value));
        }
    }

    #[test]
    fn test_write_raw_entry_powershell_output_cases() {
        let cases = [
            ("KEY", "plain", "$env:KEY = \"plain\"\n"),
            (
                "KEY",
                "value \"$HOME\" and `tick`",
                "$env:KEY = \"value `\"`$HOME`\" and ``tick``\"\n",
            ),
            ("KEY", "", "$env:KEY = \"\"\n"),
        ];

        for (key, value, expected) in cases {
            assert_eq!(
                expected,
                render_entry(RawOutputFormat::PowerShell, key, value)
            );
        }
    }

    #[test]
    #[should_panic(expected = "nu output is rendered as a single record")]
    fn test_write_raw_entry_nu_panics() {
        let _ = render_entry(RawOutputFormat::Nu, "KEY", "value");
    }

    #[test]
    fn test_write_nu_record_output_plain_case() {
        let envs = vec![
            EnvironmentRow::from("dev", "API_KEY", "value1"),
            EnvironmentRow::from("dev", "DATABASE_URL", "postgres://localhost:5432/db"),
        ];

        assert_eq!(
            "{\"API_KEY\": \"value1\", \"DATABASE_URL\": \"postgres://localhost:5432/db\"}\n",
            render_nu_record(&envs)
        );
    }

    #[test]
    fn test_write_nu_record_output_escaped_case() {
        let envs = vec![
            EnvironmentRow::from("dev", "K\"EY", "line1\nline2\t\"x\"\\y"),
            EnvironmentRow::from("dev", "NORMAL", "plain"),
        ];

        assert_eq!(
            "{\"K\\\"EY\": \"line1\\nline2\\t\\\"x\\\"\\\\y\", \"NORMAL\": \"plain\"}\n",
            render_nu_record(&envs)
        );
    }
}
