import re

INPUT = "/Users/wendy/pro/voice2text/youtube_video.txt"
OUTPUT = "/Users/wendy/pro/voice2text/youtube_video_dialogue.txt"

NAMES = {"1": "小俊", "2": "姚舜禹"}

pattern = re.compile(r"^\[说话人(\S+?)\]\s+\[[\d.]+s\s*-\s*[\d.]+s\]\s*(.*)$")

merged = []
with open(INPUT, "r", encoding="utf-8") as f:
    for line in f:
        m = pattern.match(line.rstrip("\n"))
        if not m:
            continue
        spk, text = m.group(1), m.group(2).strip()
        if not text:
            continue
        if merged and merged[-1][0] == spk:
            sep = "" if re.search(r"[，。！？、,.!?…\s]$", merged[-1][1]) else ""
            merged[-1][1] = merged[-1][1] + sep + text
        else:
            merged.append([spk, text])

with open(OUTPUT, "w", encoding="utf-8") as f:
    for spk, text in merged:
        name = NAMES.get(spk, f"说话人{spk}")
        f.write(f"{name}：{text}\n\n")

print(f"已写入 {len(merged)} 段对话到 {OUTPUT}")
