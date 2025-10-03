# Jeers

Reels-like video generator for learning japanese.

## Pipe

* [x] Get theme from user (grammar topic or new word(s))
* [x] Generate content plan with LLM
* [X] Pick random video (game-virus/japanese)
* [X] Generate audio from content plan
* [X] Cut/loop video
* [X] Insert audio to video
* [ ] Add background music to video
* [ ] Generate subs from content plan
* [ ] Insert subs to video

## TTS

Install:

```bash
git clone https://github.com/EndlessReform/fish-speech.rs.git
cargo build --release --bin server
cargo build --release --bin llama_generate
cargo build --release --bin vocoder
cargo build --release --bin encoder
cargo run --release --bin server -- --voice-dir voices-template
encoder -i ja_speaker_0.wav --checkpoint ~/.cache/huggingface/hub/models--jkeisling--fish-speech-1.5/snapshots/ef7b06eb56024dd21147fc31723ec9f9f1679f08
```

Run:

```bash
llama_generate --text "Сегодня разберём три прилагательных, которые описывают вкус: 甘い, 辛い и 塩辛い. 甘い означает «сладкий». Например: «Этот торт сладкий» — このケーキは甘いです。 辛い — это «острый» или «пряный». Так говорят о еде с перцем или васаби: «Этот суп острый» — このスープは辛いです。 塩辛い — «солёный», часто с оттенком «слишком солёный». Например: «Эта рыба слишком солёная» — この魚は塩辛いです。 Возможно вам поможет в запоминании, что 塩辛い, это 辛い с приставкой 'しお'. Запомните: 甘い — сладкий, 辛い — острый, 塩辛い — солёный. И повторим еще разок: 甘い — сладкий, 辛い — острый, 塩辛い — солёный. Представьте: вы с другом пробуете блюда в японском ресторане. Вы берёте кусочек торта — и говорите: «Вау, он очень сладкий!» — 甘い！ Потом ваш друг ест карри и морщится: «Ого, как остро!» — 辛い！ А когда вы пробуете маринованную рыбу, то сразу пьёте воду и говорите: «Это слишком солёно!» — 塩辛い！   Обратите внимание: 辛い может означать не только «острый», но и «трудный» в других контекстах, но сейчас мы говорим именно о вкусе. А 塩辛い почти всегда относится к солёному вкусу, особенно когда соли слишком много. Таким образом: 甘い — сладкий, 辛い — острый, 塩辛い — солёный" --out-path tokens.npy --checkpoint ~/.cache/huggingface/hub/models--jkeisling--fish-speech-1.5/snapshots/ef7b06eb56024dd21147fc31723ec9f9f1679f08 --max-new-tokens 4096 --prompt-tokens fake.npy --prompt-text "話せることはたくさんありますが、おそらくこれに似ていると思います" ; vocoder -i tokens.npy -o output.wav --checkpoint ~/.cache/huggingface/hub/models--jkeisling--fish-speech-1.5/snapshots/ef7b06eb56024dd21147fc31723ec9f9f1679f08
output.wav
```
