import { invoke } from "@tauri-apps/api/core";

interface Settings {
  app_id: string;
  access_token: string;
  resource_id: string;
  language: string;
  transcribe_provider: string;
  dashscope_api_key: string;
  dashscope_base_url: string;
  qwen_asr_model: string;
  qwen_asr_language: string;
  omni_prompt: string;
  refine_enabled: boolean;
  refine_api_key: string;
  refine_base_url: string;
  refine_model: string;
  refine_prompt: string;
}

interface Defaults {
  refine_prompt: string;
  refine_model: string;
  refine_base_url: string;
}

interface Meta {
  hotkey: string;
  platform: string;
  accessibility_ok: boolean;
  settings: Settings;
  defaults: Defaults;
}

let defaults: Defaults | null = null;

const $ = <T extends HTMLElement = HTMLElement>(id: string) => document.getElementById(id) as T;

async function load() {
  const meta = await invoke<Meta>("get_meta");
  $("platform").textContent =
    meta.platform === "macos" ? "macOS" : meta.platform === "windows" ? "Windows" : meta.platform;
  $("hotkey").textContent = meta.hotkey;

  if (meta.platform === "macos") {
    $("perm").textContent = meta.accessibility_ok ? "已授权" : "未授权";
    ($("grant") as HTMLButtonElement).style.display = meta.accessibility_ok ? "none" : "";
  } else {
    $("perm-row").style.display = "none";
  }

  ($("transcribe_provider") as HTMLSelectElement).value = meta.settings.transcribe_provider;
  ($("app_id") as HTMLInputElement).value = meta.settings.app_id;
  ($("access_token") as HTMLInputElement).value = meta.settings.access_token;
  ($("resource_id") as HTMLInputElement).value = meta.settings.resource_id;
  ($("language") as HTMLInputElement).value = meta.settings.language;
  ($("dashscope_api_key") as HTMLInputElement).value = meta.settings.dashscope_api_key;
  ($("dashscope_base_url") as HTMLInputElement).value = meta.settings.dashscope_base_url;
  ($("qwen_asr_model") as HTMLInputElement).value = meta.settings.qwen_asr_model;
  ($("qwen_asr_language") as HTMLInputElement).value = meta.settings.qwen_asr_language;
  ($("omni_prompt") as HTMLTextAreaElement).value = meta.settings.omni_prompt;
  ($("refine_enabled") as HTMLInputElement).checked = meta.settings.refine_enabled;
  ($("refine_api_key") as HTMLInputElement).value = meta.settings.refine_api_key;
  ($("refine_base_url") as HTMLInputElement).value = meta.settings.refine_base_url;
  ($("refine_model") as HTMLInputElement).value = meta.settings.refine_model;
  ($("refine_prompt") as HTMLTextAreaElement).value = meta.settings.refine_prompt;
  defaults = meta.defaults;
  syncProviderFields();
}

function syncProviderFields() {
  const provider = ($("transcribe_provider") as HTMLSelectElement).value;
  const isQwen = provider.startsWith("qwen");
  const isOmni = provider.startsWith("qwen35_omni");
  const isQwenAsr = provider === "qwen3_asr_flash";

  ($("qwen-card") as HTMLElement).style.display = isQwen ? "" : "none";
  ($("volc-card") as HTMLElement).style.display = provider === "volc_openspeech" ? "" : "none";
  document
    .querySelectorAll<HTMLElement>(".omni-row")
    .forEach((el) => (el.style.display = isOmni ? "" : "none"));
  document
    .querySelectorAll<HTMLElement>(".qwen-asr-row")
    .forEach((el) => (el.style.display = isQwenAsr ? "" : "none"));
}

$("grant").addEventListener("click", async () => {
  await invoke("request_accessibility");
  setTimeout(load, 400);
});

$("transcribe_provider").addEventListener("change", syncProviderFields);

$("refine_reset").addEventListener("click", () => {
  if (!defaults) return;
  ($("refine_prompt") as HTMLTextAreaElement).value = defaults.refine_prompt;
  ($("refine_model") as HTMLInputElement).value = defaults.refine_model;
  ($("refine_base_url") as HTMLInputElement).value = defaults.refine_base_url;
});

$("save").addEventListener("click", async () => {
  const s: Settings = {
    app_id: ($("app_id") as HTMLInputElement).value.trim(),
    access_token: ($("access_token") as HTMLInputElement).value.trim(),
    resource_id: ($("resource_id") as HTMLInputElement).value.trim(),
    language: ($("language") as HTMLInputElement).value.trim(),
    transcribe_provider: ($("transcribe_provider") as HTMLSelectElement).value,
    dashscope_api_key: ($("dashscope_api_key") as HTMLInputElement).value.trim(),
    dashscope_base_url: ($("dashscope_base_url") as HTMLInputElement).value.trim(),
    qwen_asr_model: ($("qwen_asr_model") as HTMLInputElement).value.trim(),
    qwen_asr_language: ($("qwen_asr_language") as HTMLInputElement).value.trim(),
    omni_prompt: ($("omni_prompt") as HTMLTextAreaElement).value.trim(),
    refine_enabled: ($("refine_enabled") as HTMLInputElement).checked,
    refine_api_key: ($("refine_api_key") as HTMLInputElement).value.trim(),
    refine_base_url: ($("refine_base_url") as HTMLInputElement).value.trim(),
    refine_model: ($("refine_model") as HTMLInputElement).value.trim(),
    refine_prompt: ($("refine_prompt") as HTMLTextAreaElement).value.trim(),
  };
  const status = $("save-status");
  try {
    await invoke("save_settings", { new: s });
    status.textContent = "已保存";
    status.className = "ok";
    setTimeout(() => (status.textContent = ""), 1500);
  } catch (e) {
    status.textContent = String(e);
    status.className = "err";
  }
});

load();
