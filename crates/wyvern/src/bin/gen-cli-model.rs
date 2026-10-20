//! Build a docli CliModel JSON file from Wyvern usage text and shipped extensions.
//!
//! This binary does not link `sc-docli`. `scripts/generate-cli-docs.sh` renders the
//! JSON with the installed `docli` 0.2.0 binary (ADR-0027, REQ-0150).

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use serde::{Deserialize, Serialize};

use wyvern::extensions::{
    extensions_usage_message, ExtensionDef, ExtensionRegistry, MatchSpec, SHIPPED_EXTENSIONS_JSON,
};
use wyvern::{browsers_usage_message, examples_usage_message, usage_message, wizard_usage_message};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("gen-cli-model: {err}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<(), ModelError> {
    let output = parse_output_arg()?;
    let usage = usage_message();
    let registry = ExtensionRegistry::from_json_str(SHIPPED_EXTENSIONS_JSON)?;
    let model = build_cli_model(&usage, &registry);
    write_model(&output, &model)
}

fn parse_output_arg() -> Result<PathBuf, ModelError> {
    let mut args = std::env::args().skip(1);
    match (args.next().as_deref(), args.next()) {
        (Some("--output"), Some(path)) if args.next().is_none() && !path.is_empty() => {
            Ok(PathBuf::from(path))
        }
        _ => Err(ModelError::Usage(
            "usage: gen-cli-model --output <path>".to_owned(),
        )),
    }
}

fn build_cli_model(usage: &str, registry: &ExtensionRegistry) -> CliModel {
    let parsed = parse_help(usage);
    let mut options = parsed.options;
    push_flag(
        &mut options,
        "help",
        "--help",
        Some("-h"),
        "Print usage and exit",
    );
    push_flag(
        &mut options,
        "version",
        "--version",
        Some("-V"),
        "Print the version and exit",
    );
    let arguments = if parsed.arguments.is_empty() {
        vec![ArgumentSpec {
            name: "input".to_owned(),
            help: "JSON command string, a path handled by an extension, or omit to read JSON from stdin.".to_owned(),
            required: false,
            default_value: None,
            choices: Vec::new(),
        }]
    } else {
        parsed.arguments
    };
    let mut subcommands = vec![
        command_from_usage("browsers", &browsers_usage_message()),
        command_from_usage("extensions", &extensions_usage_message()),
        command_from_usage("examples", &examples_usage_message()),
        command_from_usage("wizard", &wizard_usage_message()),
    ];
    for extension in registry.extensions() {
        subcommands.push(extension_command(extension));
    }
    let description = env!("CARGO_PKG_DESCRIPTION").to_owned();
    CliModel {
        name: "wyvern".to_owned(),
        version: env!("CARGO_PKG_VERSION").to_owned(),
        long_description: long_description_from(&parsed.epilogue, &description),
        description,
        usage: parsed.usage,
        epilogue: parsed.epilogue,
        options,
        arguments,
        subcommands,
    }
}

fn command_from_usage(name: &str, text: &str) -> CliModel {
    let parsed = parse_help(text);
    let (description, long_description) = describe_commands(&parsed);
    let mut subcommands: Vec<CliModel> = parsed
        .commands
        .iter()
        .map(|command| child_command(name, command))
        .collect();
    let arguments = if subcommands.len() == 1 && !parsed.arguments.is_empty() {
        subcommands[0].arguments.clone_from(&parsed.arguments);
        Vec::new()
    } else {
        parsed.arguments
    };
    CliModel {
        name: name.to_owned(),
        version: String::new(),
        description,
        long_description,
        usage: parsed.usage,
        epilogue: parsed.epilogue,
        options: parsed.options,
        arguments,
        subcommands,
    }
}

fn child_command(parent: &str, command: &CommandLine) -> CliModel {
    let usage = if command.args_hint.is_empty() {
        format!("wyvern {parent} {}", command.name)
    } else {
        format!("wyvern {parent} {} {}", command.name, command.args_hint)
    };
    CliModel {
        name: command.name.clone(),
        version: String::new(),
        description: command.help.clone(),
        long_description: String::new(),
        usage,
        epilogue: String::new(),
        options: Vec::new(),
        arguments: hint_arguments(&command.args_hint, &command.help),
        subcommands: Vec::new(),
    }
}

fn extension_command(extension: &ExtensionDef) -> CliModel {
    let description = extension.description.clone().unwrap_or_default();
    let usage = extension
        .examples
        .first()
        .cloned()
        .unwrap_or_else(|| format!("wyvern {}", extension.id));
    let mut epilogue = String::new();
    if extension.examples.len() > 1 {
        epilogue.push_str("Examples:\n");
        for example in &extension.examples {
            epilogue.push_str(example);
            epilogue.push('\n');
        }
    }
    let match_line = format_match(&extension.match_spec);
    if !match_line.is_empty() {
        epilogue.push_str("Match: ");
        epilogue.push_str(&match_line);
        epilogue.push('\n');
    }
    CliModel {
        name: extension.id.to_string(),
        version: String::new(),
        long_description: description.clone(),
        description,
        usage,
        epilogue,
        options: Vec::new(),
        arguments: Vec::new(),
        subcommands: Vec::new(),
    }
}

fn format_match(spec: &MatchSpec) -> String {
    let mut parts = Vec::new();
    if let Some(prefix) = &spec.argv_prefix {
        let tokens: Vec<&str> = prefix.iter().map(|token| token.as_str()).collect();
        parts.push(format!("argv_prefix={}", tokens.join(" ")));
    }
    if let Some(suffix) = &spec.positional_suffix {
        parts.push(format!("positional_suffix={suffix}"));
    }
    if let Some(filename) = &spec.filename {
        parts.push(format!("filename={filename}"));
    }
    if let Some(suffix) = &spec.arg_suffix {
        parts.push(format!("arg_suffix={suffix}"));
    }
    parts.join(", ")
}

fn describe_commands(parsed: &ParsedHelp) -> (String, String) {
    if parsed.commands.is_empty() {
        let description = parsed
            .epilogue
            .lines()
            .find(|line| !line.trim().is_empty())
            .unwrap_or("")
            .to_owned();
        return (description.clone(), description);
    }
    let description = parsed.commands[0].help.clone();
    let long_description = parsed
        .commands
        .iter()
        .map(|command| format!("{} — {}", command.name, command.help))
        .collect::<Vec<_>>()
        .join("\n");
    (description, long_description)
}

fn long_description_from(epilogue: &str, fallback: &str) -> String {
    epilogue
        .find("Pass a JSON")
        .map(|index| epilogue[index..].trim().to_owned())
        .filter(|text| !text.is_empty())
        .unwrap_or_else(|| fallback.to_owned())
}

fn push_flag(
    options: &mut Vec<OptionSpec>,
    name: &str,
    long: &str,
    short: Option<&str>,
    help: &str,
) {
    if options.iter().any(|option| option.name == name) {
        return;
    }
    options.push(OptionSpec {
        name: name.to_owned(),
        long: Some(long.to_owned()),
        short: short.map(str::to_owned),
        help: help.to_owned(),
        long_help: String::new(),
        value_name: None,
        required: false,
        default_value: None,
        choices: Vec::new(),
        min_values: None,
        max_values: None,
    });
}

fn write_model(path: &Path, model: &CliModel) -> Result<(), ModelError> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    let mut text = serde_json::to_string_pretty(model)?;
    text.push('\n');
    std::fs::write(path, text)?;
    Ok(())
}

#[derive(Debug)]
enum ModelError {
    Registry(wyvern::extensions::ExtensionError),
    Io(std::io::Error),
    Json(serde_json::Error),
    Usage(String),
}

impl std::fmt::Display for ModelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Registry(err) => write!(f, "extension registry: {err}"),
            Self::Io(err) => write!(f, "io: {err}"),
            Self::Json(err) => write!(f, "json: {err}"),
            Self::Usage(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for ModelError {}

impl From<wyvern::extensions::ExtensionError> for ModelError {
    fn from(err: wyvern::extensions::ExtensionError) -> Self {
        Self::Registry(err)
    }
}

impl From<std::io::Error> for ModelError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<serde_json::Error> for ModelError {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct CliModel {
    name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    version: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    description: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    long_description: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    usage: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    epilogue: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    options: Vec<OptionSpec>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    arguments: Vec<ArgumentSpec>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    subcommands: Vec<CliModel>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct OptionSpec {
    name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    long: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    short: Option<String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    help: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    long_help: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    value_name: Option<String>,
    #[serde(default)]
    required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    default_value: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    choices: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    min_values: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    max_values: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ArgumentSpec {
    name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    help: String,
    #[serde(default)]
    required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    default_value: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    choices: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CommandLine {
    name: String,
    args_hint: String,
    help: String,
}

#[derive(Debug, Default)]
struct ParsedHelp {
    usage: String,
    options: Vec<OptionSpec>,
    arguments: Vec<ArgumentSpec>,
    commands: Vec<CommandLine>,
    epilogue: String,
}

fn parse_help(text: &str) -> ParsedHelp {
    let lines: Vec<&str> = text.lines().collect();
    let mut index = 0;
    let mut usage_lines = Vec::new();
    if lines
        .first()
        .is_some_and(|line| line.trim_start().starts_with("Usage:"))
    {
        usage_lines.push(lines[0].to_owned());
        index = 1;
        while index < lines.len() {
            let line = lines[index];
            if line.trim().is_empty() || is_section_header(line) {
                break;
            }
            usage_lines.push(line.to_owned());
            index += 1;
        }
    }
    let mut parsed = ParsedHelp {
        usage: normalize_usage(&usage_lines),
        ..ParsedHelp::default()
    };
    while index < lines.len() {
        let line = lines[index];
        if line.trim().is_empty() {
            append_loose(&mut parsed.epilogue, "");
            index += 1;
            continue;
        }
        if is_section_header(line) {
            let header = line.trim().to_owned();
            index += 1;
            let mut body = Vec::new();
            while index < lines.len()
                && !lines[index].trim().is_empty()
                && !is_section_header(lines[index])
            {
                body.push(lines[index].to_owned());
                index += 1;
            }
            append_section(&mut parsed, &header, &body);
            continue;
        }
        append_loose(&mut parsed.epilogue, line);
        index += 1;
    }
    parsed.epilogue = parsed.epilogue.trim().to_owned();
    parsed
}

fn is_section_header(line: &str) -> bool {
    let trimmed = line.trim();
    !line.starts_with(' ') && trimmed.ends_with(':') && !trimmed.contains("://")
}

fn normalize_usage(lines: &[String]) -> String {
    lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                return None;
            }
            if index == 0 {
                Some(
                    trimmed
                        .strip_prefix("Usage:")
                        .unwrap_or(trimmed)
                        .trim()
                        .to_owned(),
                )
            } else {
                Some(trimmed.to_owned())
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn append_section(parsed: &mut ParsedHelp, header: &str, lines: &[String]) {
    if header.starts_with("Options") {
        parsed.options.extend(options_from_lines(lines));
        return;
    }
    if header.starts_with("Commands") {
        parsed.commands.extend(commands_from_lines(lines));
        return;
    }
    if header.starts_with("Arguments") {
        parsed
            .arguments
            .extend(arguments_from_lines(lines, &parsed.usage));
        return;
    }
    if !parsed.epilogue.is_empty() && !parsed.epilogue.ends_with('\n') {
        parsed.epilogue.push('\n');
    }
    parsed.epilogue.push_str(header);
    parsed.epilogue.push('\n');
    for line in lines {
        parsed.epilogue.push_str(line);
        parsed.epilogue.push('\n');
    }
}

fn append_loose(epilogue: &mut String, line: &str) {
    if line.is_empty() {
        if !epilogue.is_empty() && !epilogue.ends_with('\n') {
            epilogue.push('\n');
        }
        epilogue.push('\n');
        return;
    }
    epilogue.push_str(line);
    epilogue.push('\n');
}

fn options_from_lines(lines: &[String]) -> Vec<OptionSpec> {
    aligned_entries(lines)
        .into_iter()
        .filter(|entry| entry.left.starts_with('-'))
        .map(|entry| option_from_left(&entry.left, &entry.right))
        .collect()
}

fn commands_from_lines(lines: &[String]) -> Vec<CommandLine> {
    aligned_entries(lines)
        .into_iter()
        .filter(|entry| !entry.left.starts_with('-') && !entry.left.starts_with('<'))
        .map(|entry| command_from_left(&entry.left, &entry.right))
        .collect()
}

fn arguments_from_lines(lines: &[String], usage: &str) -> Vec<ArgumentSpec> {
    aligned_entries(lines)
        .into_iter()
        .filter(|entry| entry.left.starts_with('<'))
        .map(|entry| argument_from_left(&entry.left, &entry.right, usage))
        .collect()
}

struct Aligned {
    left: String,
    right: String,
}

fn aligned_entries(lines: &[String]) -> Vec<Aligned> {
    let mut entries: Vec<Aligned> = Vec::new();
    for line in lines {
        let Some(rest) = line.strip_prefix("  ") else {
            continue;
        };
        if rest.starts_with(' ') {
            if let Some(last) = entries.last_mut() {
                let extra = rest.trim();
                if extra.is_empty() {
                    continue;
                }
                if !last.right.is_empty() {
                    last.right.push(' ');
                }
                last.right.push_str(extra);
            }
            continue;
        }
        if let Some((left, right)) = split_columns(rest) {
            entries.push(Aligned {
                left: left.to_owned(),
                right: right.to_owned(),
            });
        } else if !rest.trim().is_empty() {
            entries.push(Aligned {
                left: rest.trim().to_owned(),
                right: String::new(),
            });
        }
    }
    entries
}

fn split_columns(rest: &str) -> Option<(&str, &str)> {
    let bytes = rest.as_bytes();
    let mut index = 0;
    while index + 1 < bytes.len() {
        if bytes[index] == b' ' && bytes[index + 1] == b' ' {
            let left = rest[..index].trim();
            let right = rest[index..].trim();
            if !left.is_empty() {
                return Some((left, right));
            }
        }
        index += 1;
    }
    None
}

fn option_from_left(left: &str, help: &str) -> OptionSpec {
    let mut parts = left.split_whitespace();
    let flag = parts.next().unwrap_or(left);
    let value = parts.next();
    let (name, long, short) = if let Some(name) = flag.strip_prefix("--") {
        (name.to_owned(), Some(flag.to_owned()), None)
    } else if flag.starts_with('-') && flag.len() == 2 {
        (
            flag.trim_start_matches('-').to_owned(),
            None,
            Some(flag.to_owned()),
        )
    } else {
        (flag.to_owned(), Some(flag.to_owned()), None)
    };
    let value_name = value.and_then(|token| {
        token
            .strip_prefix('<')
            .and_then(|inner| inner.strip_suffix('>'))
            .map(str::to_owned)
    });
    OptionSpec {
        name,
        long,
        short,
        help: help.to_owned(),
        long_help: String::new(),
        value_name,
        required: false,
        default_value: default_from_help(help),
        choices: choices_from_help(help),
        min_values: None,
        max_values: None,
    }
}

fn command_from_left(left: &str, help: &str) -> CommandLine {
    let mut parts = left.split_whitespace();
    let name = parts.next().unwrap_or(left).to_owned();
    let args_hint = parts.collect::<Vec<_>>().join(" ");
    CommandLine {
        name,
        args_hint,
        help: help.to_owned(),
    }
}

fn argument_from_left(left: &str, help: &str, usage: &str) -> ArgumentSpec {
    let inner = left.trim().trim_start_matches('[').trim_start_matches('<');
    let name = inner
        .split_whitespace()
        .next()
        .unwrap_or(inner)
        .trim_end_matches(']')
        .trim_end_matches('>')
        .trim_end_matches('.')
        .to_owned();
    ArgumentSpec {
        required: usage_requires(usage, &name),
        name,
        help: help.to_owned(),
        default_value: None,
        choices: Vec::new(),
    }
}

fn usage_requires(usage: &str, name: &str) -> bool {
    let required_form = format!("<{name}>");
    let optional_form = format!("[<{name}>]");
    usage.contains(&required_form) && !usage.contains(&optional_form)
}

fn hint_arguments(hint: &str, help: &str) -> Vec<ArgumentSpec> {
    let mut arguments: Vec<ArgumentSpec> = Vec::new();
    let mut rest = hint;
    while let Some(start) = rest.find('<') {
        let Some(end) = rest[start..].find('>') else {
            break;
        };
        let raw = &rest[start + 1..start + end];
        let name = raw.trim_end_matches('.').to_owned();
        let required = !rest[..start].ends_with('[');
        if !name.is_empty() && arguments.iter().all(|argument| argument.name != name) {
            arguments.push(ArgumentSpec {
                name,
                help: help.to_owned(),
                required,
                default_value: None,
                choices: Vec::new(),
            });
        }
        rest = &rest[start + end + 1..];
    }
    arguments
}

fn choices_from_help(help: &str) -> Vec<String> {
    let Some(first) = help.split_whitespace().next() else {
        return Vec::new();
    };
    if !first.contains('|') || first.contains('/') {
        return Vec::new();
    }
    first
        .split('|')
        .filter(|choice| !choice.is_empty())
        .map(str::to_owned)
        .collect()
}

fn default_from_help(help: &str) -> Option<String> {
    let lower = help.to_ascii_lowercase();
    let index = lower.find("default")?;
    let rest = help[index + "default".len()..].trim_start();
    let rest = rest.trim_start_matches(':').trim_start();
    let end = rest.find([')', ';', ',']).unwrap_or(rest.len());
    let value = rest[..end].trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_aligned_option_continuation_default_and_choices() {
        let text = concat!(
            "Usage: wyvern [OPTIONS]\n",
            "\n",
            "Options:\n",
            "  --bind <ADDR:PORT>         HTTP bind (default 127.0.0.1:0)\n",
            "  --viewer <MODE>            embedded|none|firefox\n",
            "                             (default: embedded; CI: WYVERN_VIEWER=none)\n",
        );
        let parsed = parse_help(text);
        assert_eq!(parsed.usage, "wyvern [OPTIONS]");
        assert_eq!(parsed.options.len(), 2);
        assert_eq!(parsed.options[0].value_name.as_deref(), Some("ADDR:PORT"));
        assert_eq!(
            parsed.options[0].default_value.as_deref(),
            Some("127.0.0.1:0")
        );
        assert_eq!(
            parsed.options[1].choices,
            vec!["embedded", "none", "firefox"]
        );
        assert_eq!(parsed.options[1].default_value.as_deref(), Some("embedded"));
        assert!(
            parsed.options[1].help.contains("CI:"),
            "{}",
            parsed.options[1].help
        );
    }

    #[test]
    fn wyvern_model_covers_usage_builtins_and_extension_ids() {
        let registry = ExtensionRegistry::from_json_str(SHIPPED_EXTENSIONS_JSON).expect("shipped");
        let model = build_cli_model(&usage_message(), &registry);
        assert_eq!(model.name, "wyvern");
        assert_eq!(model.version, env!("CARGO_PKG_VERSION"));
        assert!(model.usage.contains("wyvern extensions list|show"));
        let bind = model
            .options
            .iter()
            .find(|option| option.name == "bind")
            .expect("bind");
        assert_eq!(bind.default_value.as_deref(), Some("127.0.0.1:0"));
        let viewer = model
            .options
            .iter()
            .find(|option| option.name == "viewer")
            .expect("viewer");
        assert!(viewer.choices.iter().any(|choice| choice == "embedded"));
        assert!(viewer.choices.iter().any(|choice| choice == "firefox"));
        assert!(model.options.iter().any(|option| option.name == "help"));
        assert!(model.options.iter().any(|option| option.name == "version"));
        assert!(model
            .arguments
            .iter()
            .any(|argument| argument.name == "input"));
        assert!(model.epilogue.contains("Catalog ids"));
        assert!(model.long_description.contains("Pass a JSON"));

        let names: Vec<&str> = model
            .subcommands
            .iter()
            .map(|command| command.name.as_str())
            .collect();
        for builtin in ["browsers", "extensions", "examples", "wizard"] {
            assert!(names.contains(&builtin), "{builtin} missing from {names:?}");
        }
        for extension in registry.extensions() {
            assert!(
                names.iter().any(|name| *name == extension.id.as_str()),
                "missing {}",
                extension.id
            );
        }
        let mut unique = names.clone();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(unique.len(), names.len());

        let wizard = model
            .subcommands
            .iter()
            .find(|command| command.name == "wizard")
            .expect("wizard");
        assert!(wizard.epilogue.contains("WIZARD-LINT-001"));
        let lint = wizard
            .subcommands
            .iter()
            .find(|command| command.name == "lint")
            .expect("lint");
        assert!(lint
            .arguments
            .iter()
            .any(|argument| argument.name == "path"));

        let guide = model
            .subcommands
            .iter()
            .find(|command| command.name == "guide")
            .expect("guide");
        assert_eq!(guide.usage, "wyvern guide");
    }

    #[test]
    fn vendored_docli_contract_example_parses() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/cli/cli-model.schema.json");
        let text = std::fs::read_to_string(&path).expect("schema example");
        let model: CliModel = serde_json::from_str(&text).expect("parse contract example");
        assert_eq!(model.name, "demo");
        assert_eq!(model.subcommands.len(), 2);
        assert_eq!(model.options[1].choices, vec!["json", "yaml"]);
    }

    #[test]
    fn write_model_round_trips() {
        let registry = ExtensionRegistry::from_json_str(SHIPPED_EXTENSIONS_JSON).expect("shipped");
        let model = build_cli_model(&usage_message(), &registry);
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("model.json");
        write_model(&path, &model).expect("write");
        let text = std::fs::read_to_string(&path).expect("read");
        let parsed: CliModel = serde_json::from_str(&text).expect("parse");
        assert_eq!(parsed, model);
    }
}
