---
title: "Vocabulary in Origa"
slug: /docs/vocabulary
locale: en
meta_title: "Vocabulary in Origa — Dictionaries, Audio, Imports"
meta_description: "How vocabulary works in Origa: built-in dictionaries, audio pronunciation, creating cards from text or imports, and managing your word sets."
target_keywords: ["japanese vocabulary app", "japanese dictionary app", "japanese flashcards", "japanese word cards", "japanese vocabulary study"]
lastmod: 2026-07-23
status: ready
---

<!-- markdownlint-disable-file MD025 — frontmatter `title` is metadata, not a rendered H1; the body has a single H1 by design. -->

# Vocabulary

Vocabulary is the core of Origa. This page covers where words come from, how cards are built, and how to add your own.

## Built-in dictionaries

Origa ships with bilingual dictionaries — Japanese to your language. When you add a word, the dictionary supplies the translation, part of speech, and reading. You do not need to type translations manually unless you want to override them.

If a word is not in the dictionary, Origa skips it during batch creation. You can still add it manually with your own translation.

## Audio pronunciation

For supported words, Origa includes recorded pronunciation sourced from NHK and other native-speaker corpora. The audio plays on the card during review and when you preview a word.

Not every word has audio. Coverage is best for common vocabulary and JLPT-level words.

## Adding cards

There are four ways to create vocabulary cards. The full flow is described in [Getting started](/docs/getting-started); in short:

- **Text.** Type or paste Japanese; Origa tokenizes and looks up translations.
- **Image.** Run on-device OCR to extract Japanese text from a photo.
- **Audio.** Transcribe a `.wav` recording with on-device speech recognition.
- **Anki deck.** Import an existing `.apkg` file. Origa auto-detects the word and translation fields.

When you add multiple words at once, Origa shows a preview list. You confirm which to keep; duplicates are skipped silently.

## Pre-built word sets

Instead of building your own list, you can import curated sets aligned to common study paths:

- **JLPT levels.** Official vocabulary organized by N5 through N1.
- **Textbooks.** Minna no Nihongo and Irodori, organized lesson by lesson.
- **Apps.** Duolingo (English and Russian tracks), Migii JLPT prep sets.
- **Content.** Vocabulary extracted from specific anime, starting with Spy × Family.

You can import sets during onboarding, or later from the Sets page. Each set creates cards in your collection; you can review or remove them at any time.

## Card fields

A vocabulary card carries:

- The Japanese word or phrase
- Reading (furigana for kanji)
- Translation in your language
- Part of speech
- JLPT level (if known)
- Audio (if available)

Most fields fill automatically when the word exists in the dictionary. You can edit any field after creation.

## Managing your collection

The words screen lists every card in your collection. Filter by JLPT level, by source (imported set or self-added), or by review state (new, learning, mature). You can search by Japanese or by translation.

To remove cards, open the card detail and delete. Deletion is permanent — there is no trash bin.

## Related

- [Getting started](/docs/getting-started)
- [How lessons work](/docs/lesson)
- [Capture (OCR and speech)](/docs/capture)
