import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

interface HistoryItem {
  id: string;
  timestamp: string;
  text: string;
  duration_ms: number;
  error: string | null;
}

const list = document.getElementById("list") as HTMLUListElement;
const empty = document.getElementById("empty") as HTMLDivElement;
const statusText = document.getElementById("status-text") as HTMLSpanElement;
const dot = document.getElementById("dot") as HTMLSpanElement;

function fmtTime(iso: string): string {
  const d = new Date(iso);
  const now = new Date();
  const sameDay = d.toDateString() === now.toDateString();
  const hh = String(d.getHours()).padStart(2, "0");
  const mm = String(d.getMinutes()).padStart(2, "0");
  if (sameDay) return `${hh}:${mm}`;
  return `${d.getMonth() + 1}/${d.getDate()} ${hh}:${mm}`;
}

function fmtDur(ms: number): string {
  const s = ms / 1000;
  return s < 10 ? `${s.toFixed(1)}s` : `${Math.round(s)}s`;
}

function render(items: HistoryItem[]) {
  list.innerHTML = "";
  if (items.length === 0) {
    empty.style.display = "flex";
    list.style.display = "none";
    return;
  }
  empty.style.display = "none";
  list.style.display = "block";
  for (const it of items) {
    const li = document.createElement("li");
    li.className = "hist-item" + (it.error ? " err" : "");
    const preview = it.error ? `[失败] ${it.error}` : it.text;
    li.innerHTML = `
      <div class="hist-row1">
        <span class="hist-time">${fmtTime(it.timestamp)}</span>
        <span class="hist-dur">${fmtDur(it.duration_ms)}</span>
      </div>
      <div class="hist-text"></div>
    `;
    (li.querySelector(".hist-text") as HTMLDivElement).textContent = preview;
    li.addEventListener("click", async () => {
      if (it.error || !it.text) return;
      try {
        await invoke("copy_history_item", { id: it.id });
        li.classList.add("copied");
        setTimeout(() => li.classList.remove("copied"), 800);
      } catch (e) {
        console.error(e);
      }
    });
    const del = document.createElement("button");
    del.className = "hist-del";
    del.textContent = "×";
    del.title = "删除";
    del.addEventListener("click", async (e) => {
      e.stopPropagation();
      await invoke("delete_history_item", { id: it.id });
      await refresh();
    });
    li.appendChild(del);
    list.appendChild(li);
  }
}

async function refresh() {
  const items = await invoke<HistoryItem[]>("list_history");
  render(items);
}

document.getElementById("clear")!.addEventListener("click", async () => {
  if (!confirm("清空所有历史?")) return;
  await invoke("clear_history");
  await refresh();
});

document.getElementById("settings")!.addEventListener("click", async () => {
  await invoke("open_settings_window");
  await invoke("hide_popover");
});

document.getElementById("quit")!.addEventListener("click", async () => {
  await invoke("quit_app");
});

listen("history-updated", refresh);
listen("popover-opened", refresh);
listen<boolean>("recording-state", (ev) => {
  if (ev.payload) {
    dot.classList.add("rec");
    statusText.textContent = "正在录音…";
  } else {
    dot.classList.remove("rec");
    statusText.textContent = "就绪 · 按住 Fn 说话";
  }
});

refresh();
