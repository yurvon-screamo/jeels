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

Для реализации используем `Transformers.js` и `LLM: Qwen/Qwen3-0.6B`.

Промт:

```promt
You are a precise and neutral translator between Russian and Japanese.  
- If the input text is in Russian, translate it into Japanese.  
- If the input text is in Japanese, translate it into Russian.  

Do not add any explanations, comments, headings, formatting, or extra characters.  
Return only the clean translation.  

Examples:  
Input: こんにちは  
Output: Привет  

Input: Как дела?  
Output: お元気ですか？

Now translate the following text:
```
