import PIL.Image
import os
import sys

output_dir = sys.argv[1] if len(sys.argv) > 1 else "stress_test/output"
if not os.path.exists(output_dir):
    print(f"Directory {output_dir} does not exist.")
    sys.exit(1)

for f in os.listdir(output_dir):
    if f.endswith(".jpg"):
        try:
            img = PIL.Image.open(os.path.join(output_dir, f))
            print(f"{f}: {img.size}")
        except Exception as e:
            print(f"{f}: Error {e}")
