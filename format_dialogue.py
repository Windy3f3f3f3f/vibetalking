import json

with open('/Users/wendy/Desktop/transcribe_raw.json', 'r', encoding='utf-8') as f:
    data = json.load(f)

utts = data.get('result', {}).get('utterances', [])

# 合并相邻的同一说话人发言
merged = []
for u in utts:
    spk = u.get('additions', {}).get('speaker', '?')
    text = u.get('text', '').strip()
    start = u.get('start_time', 0) / 1000
    end = u.get('end_time', 0) / 1000
    if not text:
        continue
    if merged and merged[-1]['spk'] == spk:
        merged[-1]['text'] += text
        merged[-1]['end'] = end
    else:
        merged.append({'spk': spk, 'text': text, 'start': start, 'end': end})

label = {'1': '说话人A', '2': '说话人B'}
out_lines = []
for m in merged:
    name = label.get(m['spk'], f"说话人{m['spk']}")
    ts = f"[{int(m['start']//60):02d}:{int(m['start']%60):02d}]"
    out_lines.append(f"{ts} {name}: {m['text']}")

with open('/Users/wendy/pro/voice2text/meeting.txt', 'w', encoding='utf-8') as f:
    f.write('\n\n'.join(out_lines))

print(f"已写入 {len(merged)} 段对话")
print('\n'.join(out_lines[:6]))
