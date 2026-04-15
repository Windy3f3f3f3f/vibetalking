pub const APP_ID: &str = "7236214542";
pub const ACCESS_TOKEN: &str = "MMTCwjoy_KAOIaYTY64ZpwPyEP0gV0N5";
pub const RESOURCE_ID: &str = "volc.seedasr.auc";
pub const LANGUAGE: &str = "zh-CN";

pub const SUBMIT_URL: &str = "https://openspeech.bytedance.com/api/v3/auc/bigmodel/submit";
pub const QUERY_URL: &str = "https://openspeech.bytedance.com/api/v3/auc/bigmodel/query";

pub const SAMPLE_RATE: u32 = 16_000;
pub const HISTORY_MAX: usize = 500;

#[cfg(target_os = "macos")]
pub const HOTKEY_LABEL: &str = "按住 Fn";
#[cfg(target_os = "windows")]
pub const HOTKEY_LABEL: &str = "按住 右 Alt";
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub const HOTKEY_LABEL: &str = "未支持";
