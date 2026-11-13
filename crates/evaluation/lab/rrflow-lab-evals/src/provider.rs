//! How an agent is called and what its output says about usage. One place owns this, so a
//! paired trial and a task run read a provider's output the same way.
use rrflow_evaluation_core::Result;
use serde_json::Value;
use std::collections::BTreeMap;
use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const POLL_INTERVAL: Duration = Duration::from_millis(200);

/// Tokens in, tokens out, and tool calls, from one provider's output.
pub fn usage_metrics(provider: &str, output: &str) -> (Option<u64>, Option<u64>, u64) {
    match provider {
        "claude" | "gemini" => {
            let value: Value = serde_json::from_str(output).unwrap_or(Value::Null);
            let usage = value.get("usage").or_else(|| {
                value
                    .get("stats")
                    .and_then(|v| v.get("models"))
                    .and_then(Value::as_object)
                    .and_then(|models| models.values().next())
            });
            let input = usage.map(|usage| {
                [
                    "input_tokens",
                    "inputTokens",
                    "cache_creation_input_tokens",
                    "cache_read_input_tokens",
                ]
                .iter()
                .filter_map(|field| usage.get(field).and_then(Value::as_u64))
                .sum()
            });
            let output_tokens = usage
                .and_then(|u| u.get("output_tokens").or_else(|| u.get("outputTokens")))
                .and_then(Value::as_u64);
            let calls = value
                .get("num_turns")
                .and_then(Value::as_u64)
                .unwrap_or(1)
                .saturating_sub(1);
            (input, output_tokens, calls)
        }
        "codex" => {
            let values: Vec<Value> = output
                .lines()
                .filter_map(|line| serde_json::from_str(line).ok())
                .collect();
            let usage = values.iter().find_map(|value| value.get("usage"));
            let input = usage
                .and_then(|u| u.get("input_tokens"))
                .and_then(Value::as_u64);
            let output_tokens = usage
                .and_then(|u| u.get("output_tokens"))
                .and_then(Value::as_u64);
            let calls = values
                .iter()
                .filter(|value| {
                    value
                        .get("type")
                        .and_then(Value::as_str)
                        .is_some_and(|kind| {
                            kind.contains("tool")
                                || kind == "item.completed"
                                    && value.pointer("/item/type").and_then(Value::as_str)
                                        == Some("command_execution")
                        })
                })
                .count() as u64;
            (input, output_tokens, calls)
        }
        _ => (None, None, 0),
    }
}

pub struct AgentRequest<'a> {
    pub prompt: &'a str,
    pub directory: &'a Path,
    pub model: &'a str,
    pub effort: Option<&'a str>,
    pub tools: &'a [String],
    pub mcp_config: Option<&'a Path>,
    pub max_seconds: u64,
    pub max_usd: f64,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct AgentReply {
    /// The agent's final message.
    pub text: String,
    /// Everything the provider printed, kept by digest beside the run.
    pub raw: Vec<u8>,
    pub exit_code: Option<i32>,
    pub timed_out: bool,
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub cost_usd: Option<f64>,
    pub turns: Option<u64>,
    pub cache_read_tokens: Option<u64>,
    pub cache_write_tokens: Option<u64>,
    /// The model that answered, which can differ from the one asked for.
    pub model_reported: Option<String>,
    pub provider_version: Option<String>,
    /// Each MCP server's state at startup. Absent if the provider said nothing about them.
    pub mcp_servers: Option<BTreeMap<String, String>>,
    /// Calls per tool. Absent if the event stream could not be read, and empty if there were none.
    pub tool_calls: Option<BTreeMap<String, u64>>,
    /// What MCP tools replied, in order.
    pub mcp_results: Vec<String>,
}

/// A test supplies its own, so no test can start a model.
pub trait AgentProvider {
    fn run(&self, request: &AgentRequest<'_>) -> Result<AgentReply>;
}

/// The arguments for a run that sees only what its arm declares. `--bare` keeps a person's
/// memory, hooks and settings out, and needs `ANTHROPIC_API_KEY`. The prompt comes first
/// because `--tools` and `--mcp-config` take every value after them.
pub fn claude_arguments(request: &AgentRequest<'_>) -> Vec<String> {
    let mut arguments: Vec<String> = [
        "-p",
        request.prompt,
        "--bare",
        "--model",
        request.model,
        "--output-format",
        "stream-json",
        "--verbose",
        "--no-session-persistence",
        "--permission-mode",
        "plan",
        "--strict-mcp-config",
    ]
    .iter()
    .map(|text| (*text).to_string())
    .collect();
    arguments.extend(["--max-budget-usd".to_string(), request.max_usd.to_string()]);
    if let Some(effort) = request.effort {
        arguments.extend(["--effort".to_string(), effort.to_string()]);
    }
    if let Some(config) = request.mcp_config {
        arguments.extend(["--mcp-config".to_string(), config.display().to_string()]);
    }
    arguments.extend(["--tools".to_string(), request.tools.join(",")]);
    if !request.tools.is_empty() {
        arguments.extend(["--allowedTools".to_string(), request.tools.join(",")]);
    }
    arguments
}

fn content_blocks(event: &Value) -> &[Value] {
    event
        .get("message")
        .and_then(|message| message.get("content"))
        .and_then(Value::as_array)
        .map_or(&[], Vec::as_slice)
}

fn tool_result_text(block: &Value) -> String {
    match block.get("content") {
        Some(Value::String(text)) => text.clone(),
        Some(Value::Array(parts)) => parts
            .iter()
            .filter_map(|part| part.get("text").and_then(Value::as_str))
            .collect::<Vec<_>>()
            .join("\n"),
        _ => String::new(),
    }
}

/// Reads Claude Code's event stream, one event to a line. A field it cannot find stays absent
/// and is not guessed, so the capture audit can report it.
pub fn parse_claude_output(raw: Vec<u8>, exit_code: Option<i32>, timed_out: bool) -> AgentReply {
    let mut reply = AgentReply {
        exit_code,
        timed_out,
        ..AgentReply::default()
    };
    let text_output = String::from_utf8_lossy(&raw).to_string();
    let mut calls: Option<BTreeMap<String, u64>> = None;
    let mut names: BTreeMap<String, String> = BTreeMap::new();
    for line in text_output.lines() {
        let Ok(event) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        match event.get("type").and_then(Value::as_str) {
            Some("system") if event.get("subtype").and_then(Value::as_str) == Some("init") => {
                reply.provider_version = event
                    .get("claude_code_version")
                    .and_then(Value::as_str)
                    .map(str::to_string);
                reply.model_reported = event
                    .get("model")
                    .and_then(Value::as_str)
                    .map(str::to_string);
                reply.mcp_servers =
                    event
                        .get("mcp_servers")
                        .and_then(Value::as_array)
                        .map(|servers| {
                            servers
                                .iter()
                                .filter_map(|server| {
                                    Some((
                                        server.get("name")?.as_str()?.to_string(),
                                        server.get("status")?.as_str()?.to_string(),
                                    ))
                                })
                                .collect()
                        });
                calls.get_or_insert_with(BTreeMap::new);
            }
            Some("assistant") => {
                if let Some(model) = event
                    .get("message")
                    .and_then(|message| message.get("model"))
                    .and_then(Value::as_str)
                {
                    reply.model_reported = Some(model.to_string());
                }
                for block in content_blocks(&event) {
                    if block.get("type").and_then(Value::as_str) != Some("tool_use") {
                        continue;
                    }
                    let Some(name) = block.get("name").and_then(Value::as_str) else {
                        continue;
                    };
                    if let Some(calls) = calls.as_mut() {
                        *calls.entry(name.to_string()).or_default() += 1;
                    }
                    if let Some(use_id) = block.get("id").and_then(Value::as_str) {
                        names.insert(use_id.to_string(), name.to_string());
                    }
                }
            }
            Some("user") => {
                for block in content_blocks(&event) {
                    let is_mcp = block.get("type").and_then(Value::as_str) == Some("tool_result")
                        && block
                            .get("tool_use_id")
                            .and_then(Value::as_str)
                            .and_then(|use_id| names.get(use_id))
                            .is_some_and(|name| name.starts_with("mcp__"));
                    if is_mcp {
                        reply.mcp_results.push(tool_result_text(block));
                    }
                }
            }
            Some("result") => {
                let (input_tokens, output_tokens, _) = usage_metrics("claude", line);
                reply.input_tokens = input_tokens;
                reply.output_tokens = output_tokens;
                let usage = event.get("usage");
                let tokens = |field: &str| {
                    usage
                        .and_then(|usage| usage.get(field))
                        .and_then(Value::as_u64)
                };
                reply.cache_read_tokens = tokens("cache_read_input_tokens");
                reply.cache_write_tokens = tokens("cache_creation_input_tokens");
                reply.text = event
                    .get("result")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string();
                reply.cost_usd = event.get("total_cost_usd").and_then(Value::as_f64);
                reply.turns = event.get("num_turns").and_then(Value::as_u64);
            }
            _ => {}
        }
    }
    reply.tool_calls = calls;
    reply.raw = raw;
    reply
}

pub struct ClaudeCode;

impl AgentProvider for ClaudeCode {
    fn run(&self, request: &AgentRequest<'_>) -> Result<AgentReply> {
        let mut child = Command::new("claude")
            .args(claude_arguments(request))
            .current_dir(request.directory)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|error| format!("could not start claude: {error}"))?;
        let mut stdout = child.stdout.take().ok_or("claude has no standard output")?;
        let reader = std::thread::spawn(move || {
            let mut bytes = Vec::new();
            stdout.read_to_end(&mut bytes).map(|_| bytes)
        });
        let deadline = Instant::now() + Duration::from_secs(request.max_seconds);
        let mut timed_out = false;
        let status = loop {
            if let Some(status) = child.try_wait().map_err(|error| error.to_string())? {
                break status;
            }
            if Instant::now() >= deadline {
                timed_out = true;
                child.kill().map_err(|error| error.to_string())?;
                break child.wait().map_err(|error| error.to_string())?;
            }
            std::thread::sleep(POLL_INTERVAL);
        };
        let raw = reader
            .join()
            .map_err(|_| "reading claude's output failed".to_string())?
            .map_err(|error| error.to_string())?;
        Ok(parse_claude_output(raw, status.code(), timed_out))
    }
}
