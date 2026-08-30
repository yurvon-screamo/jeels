---
title: "Getting Started with Origa"
slug: /docs/getting-started
locale: en
meta_title: "Getting Started with Origa — Japanese Learning App"
meta_description: "Install Origa, sign in, choose your pace and level, import vocabulary, and complete your first lesson. Works offline after first setup."
target_keywords: ["japanese learning app getting started", "how to use origa", "japanese learning app setup", "origa first lesson", "japanese app onboarding"]
lastmod: 2026-08-30
status: ready
---

<!-- markdownlint-disable-file MD025 — frontmatter `title` is metadata, not a rendered H1; the body has a single H1 by design. -->

# Getting started

Origa is a Japanese learning app that runs on your device. This guide walks you through the first session: install, sign in, set your level, import vocabulary, and finish your first review.

Most of the setup happens once. After the first load, Origa works offline.

## 1. Install

Origa is available for Windows, Linux, macOS, and Android. A web version also runs in a browser.

[Download →](/download)

Pick the package for your platform. On desktop, the installer places Origa in your applications folder. On Android, allow the installer if the system prompts you.

## 2. Sign in

The first screen asks you to sign in. You can use a Google or Yandex account, or sign in with email and password.

There is no guest mode. An account is required because Origa syncs your learning progress across devices and remembers where you left off. A profile is created automatically the first time you sign in.

Switch the interface language on the login screen or in the first onboarding step.

## 3. Onboarding

After the first sign-in, Origa opens a short setup. You can complete it, or skip it — but without it, your first lesson will be empty because Origa has no cards to show you yet.

The setup covers four things:

- **Language.** Choose between English and Russian for the interface and translations.
- **Pace.** Pick how many new cards you want to learn per day, from minimal to maximum. You can change this later in your profile.
- **Level.** Tell Origa your current JLPT level (N5 through N1, or "unknown"). If you already know material below that level, Origa marks it as known so you don't review what you've already mastered.
- **Apps and textbooks.** If you have studied with other tools — Anki, Migii, Duolingo, Minna no Nihongo, or Irodori — select them and indicate how far you got. Origa imports the corresponding vocabulary so you don't start from zero.

At the end of the setup, Origa loads the dictionaries, kanji, grammar, and phrase data it needs. This step requires an internet connection. Once it finishes, you can study without one.

## 4. First lesson

When setup is done, the home screen shows your current JLPT progress and what is due today. Start a lesson from there.

A lesson is a sequence of cards. You see a question, reveal the answer, and rate whether you knew it. Based on your answer, the spaced-repetition scheduler decides when to show the card again.

If there are new cards for today, the lesson starts with the acquaintance stage: you browse the new group (a word with furigana and translation, kanji, grammar) and then run a short training round on it. The stage ends with a "To reviews" screen, after which the regular part of the lesson begins. Details are in [how lessons work](/docs/lesson).

The regular part uses a two-button rating — "Don't know" or "Know" (in acquaintance training it is "Remember" / "Don't remember"). Cards come in several shapes: recognition (Japanese to your language), recall (your language to Japanese), listening, writing, kanji reading, and grammar. Origa picks the mix based on what you have learned and what is due for review.

When the lesson ends, your progress syncs to the server. From here you can start another lesson or return to the dashboard.

## 5. Adding your own cards

The most direct way to study what matters to you is to add your own vocabulary. Open the words screen and choose one of four ways to create a card:

- **Text.** Type or paste a Japanese word or sentence. Origa tokenizes it, looks up the translation, and shows a preview of the cards it can create. You pick what to keep.
- **Image.** Drop or paste a photo, or pick a file. Origa runs optical character recognition on-device and extracts the Japanese text for you to review and add. The recognition model downloads the first time you use this feature.
- **Audio.** Upload a `.wav` recording. Origa transcribes it on-device using a speech recognition model and adds the recognized words. As with OCR, the model downloads on first use.
- **Anki deck.** Import an existing `.apkg` file. Origa detects the word and translation fields automatically and brings the cards in.

For all four, Origa fills in pronunciation (furigana), part of speech, and JLPT level where it can. Duplicate words are skipped.

## 6. What works offline

After the first setup, most of Origa runs without an internet connection:

- Creating cards from text, images, and audio
- Lessons and reviews
- Spaced-repetition scheduling
- Adding Anki decks
- Text-to-speech pronunciation

The internet is needed to:

- Sign in for the first time
- Load dictionaries and content on first run
- Download the OCR and speech recognition models on first use
- Sync progress across devices
- Check for app updates
- Import pre-built vocabulary sets from the catalog

If you know you will be offline for a while — a flight, a long commute — open **Profile → Offline bundle** and download everything in advance. After that, the only thing that needs the internet is account sync.

## Where to go next

- [How lessons work](/docs/lesson) — the review cycle, card types, and how Origa schedules reviews
- [Vocabulary](/docs/vocabulary) — dictionaries, audio, and importing word sets
- [Capture (OCR and speech)](/docs/capture) — how on-device recognition works and its limits
