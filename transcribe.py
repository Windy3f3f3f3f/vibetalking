import base64
import uuid
import requests
import json
import time

APP_ID = "7236214542"
ACCESS_TOKEN = "MMTCwjoy_KAOIaYTY64ZpwPyEP0gV0N5"
AUDIO_FILE = "/Users/wendy/Desktop/2026年03月10日 19点01分.mp3"

SUBMIT_URL = "https://openspeech.bytedance.com/api/v3/auc/bigmodel/submit"
QUERY_URL = "https://openspeech.bytedance.com/api/v3/auc/bigmodel/query"
RESOURCE_ID = "volc.seedasr.auc"

print("正在读取音频文件...")
with open(AUDIO_FILE, "rb") as f:
    audio_b64 = base64.b64encode(f.read()).decode("utf-8")
print(f"音频文件大小: {len(audio_b64) * 3 / 4 / 1024 / 1024:.1f} MB")

request_id = str(uuid.uuid4())

headers = {
    "Content-Type": "application/json",
    "X-Api-App-Key": APP_ID,
    "X-Api-Access-Key": ACCESS_TOKEN,
    "X-Api-Resource-Id": RESOURCE_ID,
    "X-Api-Request-Id": request_id,
    "X-Api-Sequence": "-1",
}

payload = {
    "user": {"uid": APP_ID},
    "audio": {
        "data": audio_b64,
        "format": "mp3",
        "language": "zh-CN",
    },
    "request": {
        "model_name": "bigmodel",
        "enable_itn": True,
        "enable_punc": True,
        "show_utterances": True,
        "enable_speaker_info": True,
    },
}

print(f"提交任务中... (request_id: {request_id})")
response = requests.post(SUBMIT_URL, json=payload, headers=headers, timeout=120)
status_code = response.headers.get("X-Api-Status-Code", "")
message = response.headers.get("X-Api-Message", "")
print(f"提交结果: code={status_code}, message={message}")

if status_code != "20000000":
    print(f"提交失败!")
    print(f"HTTP {response.status_code}, Body: {response.text[:500]}")
    exit(1)

print("\n任务已提交，开始轮询结果...")
query_headers = {
    "Content-Type": "application/json",
    "X-Api-App-Key": APP_ID,
    "X-Api-Access-Key": ACCESS_TOKEN,
    "X-Api-Resource-Id": RESOURCE_ID,
    "X-Api-Request-Id": request_id,
    "X-Api-Sequence": "-1",
}

for i in range(180):  # 最多等15分钟
    time.sleep(5)
    resp = requests.post(QUERY_URL, json={}, headers=query_headers, timeout=30)
    h_code = resp.headers.get("X-Api-Status-Code", "")
    h_msg = resp.headers.get("X-Api-Message", "")

    try:
        data = resp.json()
    except:
        data = {}

    # 检查 header 和 body 两种方式的状态码
    body_code = data.get("header", {}).get("code")
    code = body_code if body_code else int(h_code) if h_code.isdigit() else None

    if code == 20000000:
        text = data.get("result", {}).get("text", "")
        utterances = data.get("result", {}).get("utterances", [])

        output_file = "/Users/wendy/Desktop/2026年03月10日 19点01分.txt"
        with open(output_file, "w", encoding="utf-8") as f:
            f.write(text)
            if utterances:
                f.write("\n\n--- 分句详情 ---\n")
                for u in utterances:
                    start = u.get("start_time", 0) / 1000
                    end = u.get("end_time", 0) / 1000
                    speaker = u.get("additions", {}).get("speaker", "")
                    speaker_tag = f"[说话人{speaker}] " if speaker else ""
                    f.write(f"{speaker_tag}[{start:.1f}s - {end:.1f}s] {u.get('text', '')}\n")

        # 保存原始响应用于调试
        with open("/Users/wendy/Desktop/transcribe_raw.json", "w", encoding="utf-8") as rf:
            json.dump(data, rf, ensure_ascii=False, indent=2)

        print(f"\n识别成功！文本已保存到: {output_file}")
        print(f"文本长度: {len(text)} 字")
        if utterances:
            print(f"\n第一条utterance的所有字段: {list(utterances[0].keys())}")
            print(f"示例: {json.dumps(utterances[0], ensure_ascii=False)}")
        print(f"\n前500字预览:\n{text[:500]}")
        break
    elif code in (20000001, 20000002):
        status = "处理中" if code == 20000001 else "排队中"
        elapsed = (i + 1) * 5
        print(f"  [{elapsed}s] {status}...")
    else:
        # 如果 body 为空但 header 显示处理中
        if h_code in ("20000001", "20000002"):
            status = "处理中" if h_code == "20000001" else "排队中"
            elapsed = (i + 1) * 5
            print(f"  [{elapsed}s] {status}...")
        elif h_code == "20000000" and not data.get("result"):
            # 可能结果在 body 中但为空，继续等待
            elapsed = (i + 1) * 5
            print(f"  [{elapsed}s] 等待结果...")
        else:
            print(f"\n查询异常: header_code={h_code}, msg={h_msg}")
            print(f"Body: {json.dumps(data, ensure_ascii=False, indent=2)}")
            break
else:
    print("\n超时，任务未完成")
