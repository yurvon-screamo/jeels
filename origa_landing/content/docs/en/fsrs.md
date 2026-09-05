---
title: "How Origa Decides What to Show You"
slug: /docs/fsrs
locale: en
meta_title: "How Origa Decides What to Show You — FSRS in Plain Words"
meta_description: "The forgetting curve, spaced repetition, and FSRS: why Origa caps new cards per day, why two rating buttons instead of four, and where the review schedule comes from."
target_keywords: ["fsrs japanese", "spaced repetition japanese", "why flashcards run out"]
lastmod: 2026-08-19
status: ready
---

<!-- markdownlint-disable-file MD025 — frontmatter `title` is metadata, not a rendered H1; the body has a single H1 by design. -->

# How Origa decides what to show you

You answer a card — Origa decides when to show it again. This page is about how that decision is made: what the forgetting curve is, what spaced repetition is, and why the daily limit on new cards is part of the method, not a flaw.

## The forgetting curve

Learn a word today, and tomorrow you remember it almost certainly, in a week — barely, in a month — probably not. Ebbinghaus measured this pattern back in the 19th century, and its shape has been confirmed by many experiments since. The key fact: memory decays predictably, which means the moment of "almost forgotten" can be computed in advance.

## Spaced repetition

The idea is to show a card not every day, but at the moment you are about to forget it. A review at that moment is the strongest signal for memory, and each successful reminder makes the next interval longer: a day, then a week, then a month. So instead of flipping through the deck evenly, you review little and on time.

## FSRS

FSRS (Free Spaced Repetition Scheduler) is a modern review-scheduling algorithm — the industry standard for SRS apps since 2023, and built into Anki by default since 2024. With each answer it updates its model of your memory for that card: how well you know it and how fast you forget it. The interval to the next showing follows from the model. Origa uses FSRS as its only scheduler: it decides what to show today and what to show in a month.

## Why two buttons instead of four

In Anki you pick from four buttons after each answer: "again", "hard", "good", "easy". Origa asks one thing: **did you know the card or not**. This is a deliberate decision, not simplification for its own sake. Intermediate ratings are used meaninglessly in practice: in Anki "hard" is the most misused button, and pressing it instead of "again" distorts the model's parameters. For Japanese, the answer is most often binary: you either recalled the reading and the meaning, or you didn't. Less time choosing — more time on the language.

## Why there is a daily limit on new cards

A large batch of new cards today is not just those cards. It is many more reviews over the next two weeks, which FSRS will schedule out of those new cards. The new-card limit keeps the daily review volume within the range you actually do, rather than accumulate debt. An overloaded deck comes back as an avalanche: cards shown "on credit" return more often, and the volume grows faster than you can keep up. Hence the paces: you can change them, but each next pace has its own price in minutes per day.

## Paces and lesson size

The number of new cards per day is set by the pace in your profile settings. Six paces — from one small group of new cards per day to an intense volume; new cards are dealt in full groups of seven (the acquaintance group, see [how lessons work](/docs/lesson)), so every pace is a whole number of groups per day.

A lesson is built from the day's new cards and the cards whose review has come due — up to 22 cards. When both new cards and due reviews are exhausted, Origa says "No cards to study" — that is not an error and not an account limit: tomorrow FSRS will schedule new reviews, and lessons will appear again.

## Related

- [Lessons](/docs/lesson)
- [Getting started](/docs/getting-started)
