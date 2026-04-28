import PIL.Image
import os

output_dir = "stress_test/output"
for f in os.listdir(output_dir):
    if f.endswith(".jpg"):
        img = PIL.Image.open(os.path.join(output_dir, f))
        print(f"{f}: {img.size}")
