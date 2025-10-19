# Переводчик

Переводчик - это страница-чат, которая позволяет пользователю отправить сообщение на русском или японском языке, а ответ получить перевод.

* Отправляю японский текст в ответ получаю русский
* Отправляю русский текст в ответ получаю японский

История чата хранится в локалсторадже, чтобы я мог посмотреть что я ранее переводил.

Также пользователь должен иметь возможность:

* Отредактировать ранее отправленное сообщение и ответ на него должен быть заменен
* Удалить ранее отправленное сообщение и ответ на него должен быть удален
* "Сжечь" (очистить) всю историю

## Реализация

Для реализации используем `Transformers.js` и `Xenova/nllb-200-distilled-600M`.

Usage:

```js
import { pipeline } from '@huggingface/transformers';

// Create a translation pipeline
const translator = await pipeline('translation', 'Xenova/nllb-200-distilled-600M');

// Translate text from Hindi to French
const output = await translator('जीवन एक चॉकलेट बॉक्स की तरह है।', {
  src_lang: 'hin_Deva', // Hindi
  tgt_lang: 'fra_Latn', // French
});
console.log(output);
// [{ translation_text: 'La vie est comme une boîte à chocolat.' }]
```
