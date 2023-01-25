use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentVariable {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Property {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Platform {
    pub properties: Vec<Property>,
}

/// Details of an executed spawn.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpawnExec {
    pub command_args: Vec<String>,
    pub environment_variables: Vec<EnvironmentVariable>,
    pub platform: Platform,
    pub inputs: Vec<File>,
    pub listed_outputs: Vec<String>,
    pub remotable: bool,
    pub cacheable: bool,
    pub progress_message: String,
    pub mnemonic: String,
    pub actual_outputs: Vec<File>,
    pub runner: String,
    pub remote_cache_hit: bool,
    pub status: String,
    pub exit_code: i32,
    pub remote_cacheable: bool,
    pub target_label: String,
    pub digest: Option<Digest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct File {
    pub path: String,
    pub digest: Digest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Digest {
    pub hash: String,
    pub size_bytes: String,
    pub hash_function_name: String,
}
