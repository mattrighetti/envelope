//! Shell syntax lives here; rendering has no dependency on the database.
use std::io::Write;

use anyhow::{Result, ensure};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum RawOutputFormat {
    #[default]
    Kv,
    #[value(alias = "bash", alias = "zsh")]
    Sh,
    Fish,
    #[value(alias = "nushell")]
    Nu,
    #[value(alias = "pwsh")]
    Powershell,
}

impl RawOutputFormat {
    pub fn write<'a>(
        self,
        writer: &mut impl Write,
        entries: impl IntoIterator<Item = (&'a str, &'a str)>,
    ) -> Result<()> {
        let entries: Vec<_> = entries.into_iter().collect();
        // Validate the whole environment before emitting executable output.
        // Plain key/value output keeps accepting existing stored data unchanged.
        if self != Self::Kv {
            for &(key, value) in &entries {
                ensure!(
                    valid_name(key),
                    "shell output requires variable names matching [A-Za-z_][A-Za-z0-9_]*"
                );
                ensure!(
                    !value.contains('\0'),
                    "shell output cannot contain NUL bytes"
                );
            }
        }

        if self == Self::Nu {
            // JSON records can be consumed with `from json | load-env`.
            // Write entries in their input order to preserve --sort.
            write!(writer, "{{")?;
            for (index, &(key, value)) in entries.iter().enumerate() {
                if index > 0 {
                    write!(writer, ", ")?;
                }
                serde_json::to_writer(&mut *writer, key)?;
                write!(writer, ": ")?;
                serde_json::to_writer(&mut *writer, value)?;
            }
            writeln!(writer, "}}")?;
        } else {
            for (key, value) in entries {
                match self {
                    Self::Kv => writeln!(writer, "{key}={value}")?,
                    Self::Sh => {
                        writeln!(writer, "export {key}='{}'", value.replace('\'', "'\"'\"'"))?
                    }
                    Self::Fish => writeln!(
                        writer,
                        "set -gx {key} '{}'",
                        value.replace('\\', "\\\\").replace('\'', "\\'")
                    )?,
                    Self::Powershell => {
                        writeln!(writer, "$env:{key} = \"{}\"", quote_powershell(value))?
                    }
                    Self::Nu => unreachable!("records are rendered above"),
                }
            }
        }
        Ok(())
    }
}

fn valid_name(key: &str) -> bool {
    let mut chars = key.chars();
    chars
        .next()
        .is_some_and(|ch| ch.is_ascii_alphabetic() || ch == '_')
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

fn quote_powershell(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            // Keep each assignment on one line so text pipelines preserve
            // embedded carriage returns and newlines.
            '\r' => escaped.push_str("`r"),
            '\n' => escaped.push_str("`n"),
            // PowerShell also treats typographic double quotes as delimiters.
            '`' | '$' | '"' | '“' | '”' => {
                escaped.push('`');
                escaped.push(ch);
            }
            _ => escaped.push(ch),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(format: RawOutputFormat, entries: &[(&str, &str)]) -> String {
        let mut output = Vec::new();
        format.write(&mut output, entries.iter().copied()).unwrap();
        String::from_utf8(output).unwrap()
    }

    #[test]
    fn formats_preserve_literal_characters() {
        let cases = [
            (RawOutputFormat::Kv, "a ' $HOME", "KEY=a ' $HOME\n"),
            (
                RawOutputFormat::Sh,
                "a ' $HOME",
                "export KEY='a '\"'\"' $HOME'\n",
            ),
            (
                RawOutputFormat::Fish,
                "a\\b' $HOME",
                "set -gx KEY 'a\\\\b\\' $HOME'\n",
            ),
            (
                RawOutputFormat::Powershell,
                "\"$HOME` “x”",
                "$env:KEY = \"`\"`$HOME`` `“x`”\"\n",
            ),
        ];
        for (format, value, expected) in cases {
            assert_eq!(render(format, &[("KEY", value)]), expected);
        }
    }

    #[test]
    fn empty_values_and_environments() {
        for (format, expected) in [
            (RawOutputFormat::Kv, "KEY=\n"),
            (RawOutputFormat::Sh, "export KEY=''\n"),
            (RawOutputFormat::Fish, "set -gx KEY ''\n"),
            (RawOutputFormat::Powershell, "$env:KEY = \"\"\n"),
            (RawOutputFormat::Nu, "{\"KEY\": \"\"}\n"),
        ] {
            assert_eq!(render(format, &[("KEY", "")]), expected);
            assert_eq!(
                render(format, &[]),
                if format == RawOutputFormat::Nu {
                    "{}\n"
                } else {
                    ""
                }
            );
        }
    }

    #[test]
    fn powershell_preserves_line_endings_through_text_pipelines() {
        let output = render(
            RawOutputFormat::Powershell,
            &[("KEY", "line1\nline2\r\nline3\r`n`r\n")],
        );
        assert_eq!(output, "$env:KEY = \"line1`nline2`r`nline3`r``n``r`n\"\n");
        assert_eq!(output.lines().count(), 1);
    }

    #[test]
    fn nu_escapes_control_characters_and_preserves_sort_order() {
        let value = "line\n\r\t\u{0008}\u{001b}\\\" café";
        let output = render(RawOutputFormat::Nu, &[("Z", value), ("A", "last")]);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["Z"], value);
        assert!(output.starts_with("{\"Z\":"));
    }

    #[test]
    fn invalid_names_and_nul_fail_before_any_output() {
        for format in [
            RawOutputFormat::Sh,
            RawOutputFormat::Fish,
            RawOutputFormat::Nu,
            RawOutputFormat::Powershell,
        ] {
            for entry in [
                ("", "x"),
                ("1KEY", "x"),
                ("K; echo injected", "x"),
                ("KEY", "a\0b"),
            ] {
                let mut output = Vec::new();
                assert!(format.write(&mut output, [("OK", "first"), entry]).is_err());
                assert!(output.is_empty());
            }
        }
        assert_eq!(
            render(RawOutputFormat::Kv, &[("a.b", "value")]),
            "a.b=value\n"
        );
    }

    #[cfg(unix)]
    #[test]
    fn sh_output_round_trips_through_a_real_shell() {
        use std::process::Command;

        for value in [
            "",
            " spaces ",
            "'\"\\$HOME `uname` $(uname); &|<>% !",
            "line1\nline2\n",
            "café 🦀",
        ] {
            let script = render(RawOutputFormat::Sh, &[("ENVELOPE_TEST_VALUE", value)])
                + "printf '%s' \"$ENVELOPE_TEST_VALUE\"";
            let output = Command::new("sh").args(["-c", &script]).output().unwrap();
            assert!(output.status.success());
            assert_eq!(output.stdout, value.as_bytes());
        }
    }

    #[test]
    fn writer_errors_are_propagated() {
        for format in [
            RawOutputFormat::Kv,
            RawOutputFormat::Nu,
            RawOutputFormat::Sh,
        ] {
            let mut output = &mut [][..];
            assert!(format.write(&mut output, [("KEY", "value")]).is_err());
        }
    }
}
