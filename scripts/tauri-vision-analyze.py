#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.10"
# dependencies = [
#   "transformers>=4.40",
#   "torch>=2.0",
#   "pillow>=10.0",
#   "huggingface-hub>=0.20",
# ]
# ///
"""Analyze a screenshot using Florence-2-base local vision model.

Usage:
  uv run scripts/tauri-vision-analyze.py --image <path> --prompt "What does this window show?"

Outputs JSON to stdout with detected content summary.
"""
import argparse, base64, json, sys, os, time
from pathlib import Path

def analyze(image_path: str, prompt: str = "What does this window show?") -> dict:
    from PIL import Image
    from transformers import AutoProcessor, AutoModelForCausalLM
    import torch

    start = time.time()
    device = "cuda" if torch.cuda.is_available() else "cpu"
    model_id = "microsoft/Florence-2-base"

    sys.stderr.write(f"[vision] loading {model_id} on {device}...\n")
    model = AutoModelForCausalLM.from_pretrained(model_id, trust_remote_code=True).to(device)
    processor = AutoProcessor.from_pretrained(model_id, trust_remote_code=True)
    load_ms = int((time.time() - start) * 1000)

    sys.stderr.write(f"[vision] loaded in {load_ms}ms, processing image...\n")
    image = Image.open(image_path).convert("RGB")
    task = "<MORE_DETAILED_CAPTURE>"
    inputs = processor(text=task, images=image, return_tensors="pt").to(device)

    with torch.no_grad():
        generated_ids = model.generate(
            input_ids=inputs["input_ids"],
            pixel_values=inputs["pixel_values"],
            max_new_tokens=100,
            num_beams=3,
        )
    generated_text = processor.batch_decode(generated_ids, skip_special_tokens=False)[0]
    result = processor.post_process_generation(generated_text, task=task, image_size=image.size)
    inference_ms = int((time.time() - start) * 1000) - load_ms

    caption = result.get(task, "")
    return {
        "model": model_id,
        "device": device,
        "load_ms": load_ms,
        "inference_ms": inference_ms,
        "total_ms": load_ms + inference_ms,
        "caption": caption,
        "prompt": prompt,
        "image_path": image_path,
        "image_size": f"{image.size[0]}x{image.size[1]}",
    }

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--image", required=True)
    parser.add_argument("--prompt", default="What does this window show?")
    args = parser.parse_args()

    if not os.path.exists(args.image):
        print(json.dumps({"error": f"image not found: {args.image}"}))
        sys.exit(1)

    result = analyze(args.image, args.prompt)
    print(json.dumps(result, indent=2))
