import os
import argparse
from io import BytesIO
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

def run_reconstruction(input_path, output_path):
    """
    Performs generative macro detail reconstruction using Gemini 3.0 Pro.
    """
    print(f"Loading image: {input_path}")
    img = Image.open(input_path)
    
    # PROMPT OPTIMIZED FOR MACRO FEET TEXTURE
    macro_prompt = (
        "Extreme macro photography close-up. "
        "Ultra-high resolution skin texture, visible pores, natural fine lines, and microscopic skin details. "
        "Photorealistic, sharp focus, cinematic lighting, 8k resolution, professional macro lens aesthetic. "
        "Preserve original structure while reconstructing microscopic textures."
    )
    
    print(f"Reconstructing details with gemini-3-pro-image-preview...")
    try:
        response = client.models.generate_content(
            model="gemini-3-pro-image-preview",
            contents=[
                macro_prompt,
                img
            ]
        )
        
        # Process the multimodal response parts
        found_image = False
        for part in response.candidates[0].content.parts:
            if part.inline_data:
                # The model returns the edited/generated image as raw bytes
                reconstructed_image = Image.open(BytesIO(part.inline_data.data))
                reconstructed_image.save(output_path)
                print(f"Successfully saved reconstructed image to: {output_path}")
                found_image = True
                break
        
        if not found_image:
            print("The model did not return an image. It might have returned text instead:")
            for part in response.candidates[0].content.parts:
                if part.text:
                    print(f"Model text output: {part.text}")
                    
    except Exception as e:
        print(f"Error during reconstruction: {e}")

def main():
    parser = argparse.ArgumentParser(description="Gemini 3.0 Pro Macro Reconstructor")
    parser.add_argument("input", help="Path to input image (feet close-up).")
    parser.add_argument("--output", default="reconstructed_macro.png", help="Path for output file.")
    
    args = parser.parse_args()
    
    if not os.path.exists(args.input):
        print(f"File not found: {args.input}")
        return
        
    run_reconstruction(args.input, args.output)

if __name__ == "__main__":
    main()
