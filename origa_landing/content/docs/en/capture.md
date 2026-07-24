---
title: "Capture: OCR and Speech Recognition in Origa"
slug: /docs/capture
locale: en
meta_title: "Capture in Origa — On-Device OCR and Speech Recognition"
meta_description: "How Origa's on-device OCR and speech recognition work, when models download, supported formats, and known limits. All processing happens locally."
target_keywords: ["japanese ocr app", "japanese speech recognition", "japanese stt offline", "japanese text recognition", "ocr japanese learning"]
lastmod: 2026-07-23
status: ready
---

<!-- markdownlint-disable-file MD025 — frontmatter `title` is metadata, not a rendered H1; the body has a single H1 by design. -->

# Capture: OCR and speech recognition

Origa can read Japanese from images and transcribe it from audio. Both features run on your device — the processing never sends your photos or recordings to a server. This page covers how to use them and what their limits are.

## Optical character recognition (OCR)

OCR extracts Japanese text from images. Use it when you have a photo of a menu, a page, a sign, a screenshot, or any other visual source of Japanese text.

**How to use:**

1. Open the words screen and choose the **Image** tab.
2. Drop or paste an image, or pick a file from your device.
3. Origa runs OCR and shows the recognized text.
4. Review the extracted words, pick the ones you want, and add them as cards.

**Supported formats:** PNG, JPEG, WebP. A maximum file size applies.

**When the model downloads:** The OCR model is not bundled with the installer. It downloads the first time you use the feature, with a progress indicator. After the first download, OCR works offline.

**What OCR handles well:** printed text, clean screenshots, typed Japanese.

**Where OCR struggles:** heavily stylized fonts, low-contrast photos, vertical text set in unusual layouts, very small characters. Handwritten text is not supported.

## Speech recognition (STT)

STT transcribes spoken Japanese from an audio file. Use it when you have a recording of speech — a podcast clip, a line from a show, a voice memo.

**How to use:**

1. Open the words screen and choose the **Audio** tab.
2. Upload an audio file.
3. Origa transcribes it on-device and shows the recognized text.
4. Review and add the words you want.

**Supported format:** WAV only. Other common formats (MP3, M4A, OGG) are not currently accepted — convert your file before uploading.

**When the model downloads:** As with OCR, the STT model is not bundled. It downloads on first use, then runs offline.

**What STT handles well:** clear single-speaker audio, slow to moderate pace.

**Where STT struggles:** overlapping speakers, heavy background noise, very fast speech, strong accents. Recognition is approximate; review the output before adding words.

## Privacy

Both OCR and STT run entirely on your device. The image or audio you supply is processed locally and is never uploaded. The only network traffic is the one-time model download from Origa's CDN.

## When to use capture

Capture is not the primary way to add vocabulary — typing is faster for words you already know. Capture shines when:

- You are reading physical material (a book, a sign) and want to capture unknown words without typing.
- You have a screenshot from a manga reader or a subtitle and want to turn it into cards.
- You heard a word in audio content and want to look it up without typing what you (approximately) heard.

## Related

- [Vocabulary](/docs/vocabulary)
- [Getting started](/docs/getting-started)
- [Limitations](/docs/limitations)
