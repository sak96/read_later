#!/usr/bin/env python3
"""
translate_toml.py

Translate a flat TOML file (key = "value") from English to multiple languages
using Ollama TranslateGemma model via OpenAI-compatible endpoints.

Default source file:
    src-tauri/locales/en.toml

Default target languages:
    zh, es, ar, fr, pt, ru

Example en.toml content:
    greeting = "Hello"
    farewell = "Goodbye"
    about = "About"
    version = "Version"

This will produce <lang>.toml files in the same directory as the source file, e.g.:
    src-tauri/locales/zh.toml
    src-tauri/locales/es.toml
    src-tauri/locales/ar.toml
    ...

Usage:
    python translate_toml.py
    python translate_toml.py --file src-tauri/locales/en.toml
    python translate_toml.py --file src-tauri/locales/en.toml --languages fr de es
    python translate_toml.py --model custommodel:latest
"""

import urllib.request
import json
from pathlib import Path
import argparse
import sys

# Ollama OpenAI-compatible server
OLLAMA_URL = "http://localhost:11434/v1"

# Default target languages covering major global languages
DEFAULT_LANGUAGES = ["zh", "es", "ar", "fr", "pt", "ru"]
DEFAULT_MODEL = "translategemma:latest"


def check_model_available(model_name: str):
    """Check if the required model is available on the Ollama server via OpenAI-compatible endpoint."""
    try:
        req = urllib.request.Request(f"{OLLAMA_URL}/models", method="GET")
        with urllib.request.urlopen(req) as response:
            models_info = json.load(response)
            models = [model["id"] for model in models_info.get("data", [])]
            if model_name not in models:
                print(
                    f"ERROR: Required model '{model_name}' is not installed on the Ollama server."
                )
                print(f"Installed models: {models}")
                print(f"Install it by running: ollama pull {model_name}")
                sys.exit(1)
    except Exception as e:
        print(f"ERROR: Cannot connect to Ollama server at {OLLAMA_URL}.")
        print(f"Details: {e}")
        sys.exit(1)


def translate_toml(file_path: str, languages: list[str], model_name: str, source_code: str = "en"):
    """
    Translate a simple flat TOML file (key = "value") to multiple languages
    using Ollama TranslateGemma via OpenAI-compatible endpoints, and save <lang>.toml files
    in the same folder.

    Args:
        file_path (str): Path to the source en.toml file.
        languages (list[str]): List of target language codes, e.g., ["fr", "de", "es"].
        model_name (str): Model name on Ollama server.
        source_code (str): Source language code, default "en".
    """
    path_obj = Path(file_path)
    if not path_obj.exists():
        raise FileNotFoundError(f"{path_obj} does not exist.")

    # Read the flat TOML file into a dictionary
    data = {}
    with path_obj.open("r", encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line or line.startswith("#"):
                continue
            if "=" in line:
                key, val = line.split("=", 1)
                key = key.strip()
                val = val.strip().strip('"')
                data[key] = val

    for tgt in languages:
        translated_data = {}
        for key, text in data.items():
            # Build TranslateGemma prompt
            prompt = (
                f"You are a professional English ({source_code}) to {tgt} ({tgt}) translator. "
                "Your goal is to accurately convey the meaning and nuances of the original English text while "
                "adhering to correct grammar, vocabulary, and cultural sensitivities. "
                "Produce only the translation, without any additional explanations or commentary. "
                "Please translate the following English text into the target language:\n\n"
                f"{text}"
            )

            # Prepare OpenAI-compatible HTTP request payload
            payload = {
                "model": model_name,
                "messages": [{"role": "user", "content": prompt}],
            }
            data_bytes = json.dumps(payload).encode("utf-8")
            req = urllib.request.Request(
                url=f"{OLLAMA_URL}/chat/completions",
                data=data_bytes,
                headers={"Content-Type": "application/json"},
                method="POST",
            )

            # Send request
            with urllib.request.urlopen(req) as response:
                result = json.load(response)
                translated_text = result["choices"][0]["message"]["content"].strip()
                translated_data[key] = translated_text

        # Write translated TOML to same folder
        output_file = path_obj.parent / f"{tgt}.toml"
        with output_file.open("w", encoding="utf-8") as f:
            for k, v in translated_data.items():
                f.write(f'{k} = "{v}"\n')

        print(f"Created {output_file}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="Translate a flat TOML file using Ollama TranslateGemma."
    )
    parser.add_argument(
        "--file",
        default="src-tauri/locales/en.toml",
        help="Path to the source en.toml file (default: src-tauri/locales/en.toml)",
    )
    parser.add_argument(
        "--languages",
        nargs="+",
        default=DEFAULT_LANGUAGES,
        help=f"Target language codes (default: {DEFAULT_LANGUAGES})",
    )
    parser.add_argument(
        "--model",
        default=DEFAULT_MODEL,
        help=f"Model name on Ollama server (default: {DEFAULT_MODEL})",
    )
    args = parser.parse_args()

    # Check Ollama server and model
    check_model_available(args.model)

    # Run translation
    translate_toml(args.file, args.languages, args.model)
