---
title: "How Lessons Work in Origa"
slug: /docs/lesson
locale: en
meta_title: "How Lessons Work in Origa — Reviews, Cards, Scheduling"
meta_description: "The structure of an Origa lesson: the acquaintance stage for new cards (presentation and training), card types, rating, and review scheduling."
target_keywords: ["origa lesson", "how to learn new japanese words", "spaced repetition japanese", "fsrs japanese", "japanese review system", "japanese flashcards review"]
lastmod: 2026-08-30
status: ready
---

<!-- markdownlint-disable-file MD025 — frontmatter `title` is metadata, not a rendered H1; the body has a single H1 by design. -->

# How lessons work

A lesson in Origa is a sequence of cards chosen for this session. You answer each one, rate whether you knew it, and Origa schedules the next review. This page covers what is in a lesson, how the rating works, and how the schedule adapts.

## What is in a lesson

Each lesson pulls three kinds of cards:

- **New cards.** Drawn from your active vocabulary, kanji, grammar, or phrase sets. The number per day comes from the pace you chose during onboarding (or changed later in your profile). Vocabulary, kanji, and grammar first pass through the acquaintance stage (next section); new phrases go straight to reviews.
- **Due reviews.** Cards whose previous interval has expired. These take priority over new cards.
- **Mixed views.** The same word can appear in different shapes — recognition, recall, listening, writing — so you encounter it from multiple angles.

Origa decides the mix based on what is due, what is new, and what view types are available for each piece of content. If nothing is due and no new cards are available, the lesson is empty.

## The acquaintance stage

A new card does not appear in the lesson mixed in with reviews. It first goes through a short introductory cycle — so meeting a word for the first time does not double as a memory test.

### Presentation

The day's new cards are gathered into a small group. You go through them one by one: a word with furigana, translation, and audio; a kanji with meanings and readings; grammar with examples. Nothing to recall yet. Hovering a kanji inside a word opens a brief description of the character.

Each card has two actions: **Next** (space) and **Already know**. "Already know" marks the card as known — it goes straight into regular reviews, and its slot in the group is taken by the next card from the queue; if the queue is empty, the group simply gets smaller. The daily quota of new cards is not spent.

### Training

Once every card has been shown, the group moves to a short training round. The cards cycle through the group: you see one side, reveal the answer, and rate — **Remember** or **Don't remember**.

Words are trained in both directions: first "Japanese → your language", and once you recall it confidently — "your language → Japanese". "Don't remember" resets that card's progress: the streak of correct answers starts over. Kanji and grammar train until recall is stable.

The strip in the header shows how many cards of the group are closed. When all are closed, the "Acquaintance complete" screen appears — the "To reviews" button returns you to the regular part of the lesson.

Training writes nothing to the schedule: FSRS starts counting intervals when a card first arrives at a regular review.

## Pace and lesson size

The number of new cards per day is set by the pace in your profile settings — six values from 3 to 30. A lesson is built from the day's new cards and the reviews that have come due.

**Lessons ran out and you are ready to keep going.** That means the daily quota of new cards is spent and no reviews are due. Origa says "No cards to study" — this is not an error or an account limit: tomorrow FSRS will schedule new reviews, and lessons will appear again. If the volume is not enough — raise the pace in your profile or add a set of the next level (see "JLPT progress").

## Card types

You will meet several shapes during a lesson:

- **Recognition.** You see a Japanese word or sentence and pick or type the meaning in your language.
- **Recall.** The reverse — your language is shown, and you recall the Japanese.
- **Listening.** A phrase plays; you transcribe or pick the matching text.
- **Writing.** A kanji is shown with its stroke order animated; you follow along to learn the correct writing.
- **Kanji reading.** You read a word with hidden furigana and supply the pronunciation.
- **Grammar.** A grammar pattern is shown in context; you complete or identify it.

Not every card has every shape. A vocabulary card can be recognition or recall; a kanji card can also be writing or reading; a grammar card has grammar-specific shapes.

## Rating

Which pair of buttons you see depends on the part of the lesson: in acquaintance training it is **Remember** / **Don't remember**, in a regular review — **Don't know** / **Know**. Both are a single binary signal for the scheduler.

- **Don't remember / Don't know** schedules the card to come back soon — "Don't remember" in acquaintance training also resets that card's progress.
- **Remember / Know** moves the card forward: in training it brings the group closer to completion; in review it extends the interval before the next showing. The longer the streak, the longer the interval.

The scheduler behind this is FSRS, the same family of algorithms used by modern spaced-repetition systems. It adapts to your memory curve per card.

## What happens after a lesson

When the last card is answered, Origa shows a completion screen and syncs your progress to the server. If you are offline, the sync is queued and runs when you reconnect.

You can start another lesson immediately, or return to the dashboard. The dashboard shows what is due next and your current JLPT progress.

## JLPT progress

Every card is tagged with a JLPT level (N5 through N1). Origa shows the lowest JLPT level in which you still have gaps. New cards are issued in ascending level order (N5 → N1) the same way for everyone — the app does not diagnose "where exactly your holes are". The level indicator reflects your material, not its route. As you learn and retain cards at a level, your JLPT progress for that level rises. The dashboard reflects this so you can see where you stand.

JLPT progress is an internal estimate based on the cards you have studied. It is not an official JLPT score.

## Related

- [Getting started](/docs/getting-started)
- [Vocabulary](/docs/vocabulary)
- [Kanji](/docs/kanji)
- [Grammar](/docs/grammar)
