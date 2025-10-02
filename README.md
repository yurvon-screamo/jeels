# Jeers

Reels-like video generator for learning japanese.

## Pipe

* [x] Get theme from user (grammar topic or new word(s))
* [x] Generate content plan with LLM
* [X] Pick random video (game-virus/japanese)
* [X] Generate audio from content plan
* [ ] Cut/loop video
* [ ] Insert audio to video
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
cargo run --release --bin server
# cargo run --release --bin server -- --voice-dir voices-template
```

Run:

```bash
llama_generate --text "Привет, это тестовое сообщение без референсного голоса.　こんにちは、これは参照音声のないテストメッセージです。" --out-path tokens.npy --checkpoint ~/.cache/huggingface/hub/models--jkeisling--fish-speech-1.5/snapshots/ef7b06eb56024dd21147fc31723ec9f9f1679f08
vocoder -i tokens.npy -o output.wav --checkpoint ~/.cache/huggingface/hub/models--jkeisling--fish-speech-1.5/snapshots/ef7b06eb56024dd21147fc31723ec9f9f1679f08
output.wav
```
