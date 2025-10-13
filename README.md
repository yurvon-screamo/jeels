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
llama_generate --text "Сегодня разберём три прилагательных, которые описывают вкус: 甘い, 辛い и 塩辛い.  \
甘い означает «сладкий». Например: «Этот торт сладкий» — このケーキは甘いです。  \
辛い — это «острый» или «пряный». Так говорят о еде с перцем или васаби: «Этот суп острый» — このスープは辛いです。  \
塩辛い — «солёный», часто с оттенком «слишком солёный». Например: «Эта рыба слишком солёная» — この魚は塩辛いです。  \
Запомните: 甘い — сладкий, 辛い — острый, 塩辛い — солёный.  \
И повторим ещё раз: 甘い — сладкий, 辛い — острый, 塩辛い — солёный." --out-path tokens.npy --checkpoint ~/.cache/huggingface/hub/models--jkeisling--fish-speech-1.5/snapshots/ef7b06eb56024dd21147fc31723ec9f9f1679f08 --max-new-tokens 4096; vocoder -i tokens.npy -o output.wav --checkpoint ~/.cache/huggingface/hub/models--jkeisling--fish-speech-1.5/snapshots/ef7b06eb56024dd21147fc31723ec9f9f1679f08
output.wav
```

https://www.openai.fm/

## Fish Audio

```promt
Создай превью-логотип для урока '待ちます, 探します, 見つけます' в стиле чиби-манга в макаронных тонах
```

```promt
Создай чиби-мангу для урока в макаронных тонах 3-4 кадра:

<lesson>
# 待ちます, 探します, 見つけます

Сегодня разберём три глагола, связанных с ожиданием и поиском: 待ちます, 探します и 見つけます.  

待ちます означает «ждать». Например: «Я жду автобус» — バスを待ちます。  

探します — это «искать» что-то, что потерялось или нужно найти. Например: «Я ищу свой телефон» — わたしは携帯電話を探します。  

見つけます — «находить», то есть успешно завершить поиск. Например: «Я нашёл ключи» — わたしは鍵を見つけました。  

Запомните: 探します — это процесс поиска, а 見つけます — результат. А 待ちます — совсем другое: вы не ищете, а просто ждёте кого-то или чего-то.  

Повторим: 待ちます — «ждать», 探します — «искать», 見つけます — «находить». И ещё раз: 待ちます — «ждать», 探します — «искать», 見つけます — «находить».  

Представьте: вы пришли в парк, чтобы встретиться с другом. Вы садитесь на скамейку и ждёте — 友達を待ちます。  
Но друг не приходит. Тогда вы звоните ему и узнаёте, что он потерял ваш адрес. Он говорит: «Извини, я ищу кафе, где мы договорились встретиться» — カフェを探します。  
Через десять минут он пишет: «Нашёл! Это рядом с магазином!» — カフェを見つけました！  

В этой ситуации: вы ждали (待ちます), друг искал (探します) и в итоге нашёл (見つけます).  

Итак, ещё раз: 待ちます — «ждать», 探します — «искать», 見つけます — «находить».
</lesson>
```

### ASR

```bash
uv run whisperx Izuchim_raznitsu_mezhdu_slova.mp3 --compute_type int8 -f json
```

УДАЛИТЬ word_segments

````promt
Вот транскрибация аудио, но в ней допущена ошибка - вместо японский слов тут транслит. Надо заменить транслит на корректный японский текст, сохранив формат и русский язык.　Результат - ТОЛЬКО json.

```json
```
````
