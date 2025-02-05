use crate::native::hasher::hash;
use dashmap::DashMap;
use std::collections::HashMap;
use std::process::Command;
use std::sync::Arc;
use tracing::trace;

pub fn hash_runtime(
    workspace_root: &str,
    command: &str,
    env: &HashMap<String, String>,
    cache: Arc<DashMap<String, String>>,
) -> anyhow::Result<String> {
    let cache_key = format!("{}-{:?}", command, env);

    if let Some(cache_results) = cache.get(&cache_key) {
        return Ok(cache_results.clone());
    }

    let mut command_builder = if cfg!(target_os = "windows") {
        let comspec = std::env::var("COMSPEC");
        let shell = comspec
            .as_ref()
            .map(|v| v.as_str())
            .unwrap_or_else(|_| "cmd.exe");
        let mut command = Command::new(shell);
        command.arg("/C");
        command
    } else {
        let mut command = Command::new("sh");
        command.arg("-c");
        command
    };

    command_builder.arg(command);

    command_builder.current_dir(workspace_root);
    env.iter().for_each(|(key, value)| {
        command_builder.env(key, value);
    });
    trace!("executing: {:?}", command_builder);
    let output = command_builder
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to execute: '{}'\n{}", command, e))?;
    trace!("{} output: {:?}", command, output);

    let std_out = std::str::from_utf8(&output.stdout)?.trim();
    let std_err = std::str::from_utf8(&output.stderr)?.trim();
    let hash_result = hash(&[std_out.as_bytes(), std_err.as_bytes()].concat());

    cache.insert(cache_key, hash_result.clone());

    Ok(hash_result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use dashmap::DashMap;
    use std::collections::HashMap;
    use std::sync::Arc;

    #[test]
    fn test_hash_runtime() {
        let workspace_root = "/tmp";
        let command = "echo 'runtime'";
        let env: HashMap<String, String> = HashMap::new();
        let cache = Arc::new(DashMap::new());

        let result = hash_runtime(workspace_root, command, &env, Arc::clone(&cache)).unwrap();
        assert_eq!(result, "10571312846059850300");
    }
}
