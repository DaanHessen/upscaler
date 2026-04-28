import os
import requests
import random

input_dir = r'c:\Users\daanh\Documents\AA-GEMINI-UPSCALER-RUST-BACKEND\stress_test\input'
os.makedirs(input_dir, exist_ok=True)

# Define resolutions for testing
# 5x Low Res (< 512)
# 5x Medium Res (~1MP)
# 5x High Res (~4MP)
# 5x Ultra High Res (> 6MP)
test_configs = [
    (256, 256, 5, "low"),
    (1024, 1024, 5, "med"),
    (2048, 2048, 5, "high"),
    (3000, 2000, 5, "ultra")
]

print("Downloading test images...")
for w, h, count, label in test_configs:
    for i in range(count):
        seed = random.randint(1, 10000)
        url = f"https://picsum.photos/seed/{seed}/{w}/{h}"
        filename = f"{label}_{i+1}_{w}x{h}.jpg"
        filepath = os.path.join(input_dir, filename)
        
        print(f"Downloading {filename}...")
        try:
            response = requests.get(url, timeout=30)
            if response.status_code == 200:
                with open(filepath, 'wb') as f:
                    f.write(response.content)
            else:
                print(f"Failed to download {url}")
        except Exception as e:
            print(f"Error downloading {url}: {e}")

print("Download complete.")
