import { invoke } from "@tauri-apps/api/core";

interface Settings {
  app_id: string;
  access_token: string;
  resource_id: string;
  language: string;
}

interface Meta {
  hotkey: string;
  platform: string;
  accessibility_ok: boolean;
  settings: Settings;
}

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

  ($("app_id") as HTMLInputElement).value = meta.settings.app_id;
  ($("access_token") as HTMLInputElement).value = meta.settings.access_token;
  ($("resource_id") as HTMLInputElement).value = meta.settings.resource_id;
  ($("language") as HTMLInputElement).value = meta.settings.language;
}

$("grant").addEventListener("click", async () => {
  await invoke("request_accessibility");
  setTimeout(load, 400);
});

$("save").addEventListener("click", async () => {
  const s: Settings = {
    app_id: ($("app_id") as HTMLInputElement).value.trim(),
    access_token: ($("access_token") as HTMLInputElement).value.trim(),
    resource_id: ($("resource_id") as HTMLInputElement).value.trim(),
    language: ($("language") as HTMLInputElement).value.trim(),
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
