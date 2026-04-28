import PIL.Image
import numpy as np
import sys

def get_avg_color(path):
    img = PIL.Image.open(path).convert("RGB")
    data = np.array(img)
    avg = np.mean(data, axis=(0, 1))
    return avg

img1 = sys.argv[1]
img2 = sys.argv[2]

avg1 = get_avg_color(img1)
avg2 = get_avg_color(img2)

print(f"Original Avg RGB: {avg1}")
print(f"Upscaled Avg RGB: {avg2}")
print(f"Diff: {avg2 - avg1}")
