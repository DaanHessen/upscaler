import os
import argparse
from dotenv import load_dotenv
from PIL import Image
from google import genai
from google.genai import types

# Load configuration
load_dotenv()
PROJECT_ID = os.getenv("PROJECT_ID")
LOCATION = os.getenv("LOCATION", "us-central1")

if not PROJECT_ID:
    print("Error: PROJECT_ID not found in .env file.")
    exit(1)

# Initialize the GenAI Client for Vertex AI
client = genai.Client(vertexai=True, project=PROJECT_ID, location=LOCATION)

def run_upscale(input_path, output_path, factor="x4"):
    """
    Performs a high-fidelity upscale using the dedicated Imagen 4 model.
    """
    print(f"Loading image: {input_path}")
    img = Image.open(input_path)
    
    print(f"Upscaling with imagen-4.0-upscale-preview (Factor: {factor})...")
    try:
        response = client.models.upscale_image(
            model='imagen-4.0-upscale-preview',
            image=img,
            upscale_factor=factor,
            config=types.UpscaleImageConfig(
                output_mime_type='image/png',
            )
        )
        
        # Extract and save the result
        upscaled_image = response.generated_images[0].image
        upscaled_image.save(output_path)
        print(f"Successfully saved upscaled image to: {output_path}")
        
    except Exception as e:
        print(f"Error during upscaling: {e}")

def main():
    parser = argparse.ArgumentParser(description="Imagen 4.0 Pure Upscaler")
    parser.add_argument("input", help="Path to input image.")
    parser.add_argument("--output", default="upscaled_output.png", help="Path for output file.")
    parser.add_argument("--factor", choices=["x2", "x3", "x4"], default="x4", help="Upscale factor.")
    
    args = parser.parse_args()
    
    if not os.path.exists(args.input):
        print(f"File not found: {args.input}")
        return
        
    run_upscale(args.input, args.output, args.factor)

if __name__ == "__main__":
    main()
