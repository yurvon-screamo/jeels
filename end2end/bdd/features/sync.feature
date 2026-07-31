# language: ru
Функция: Синхронизация данных

  Сценарий: User ID не nil после входа
    Допустим новый пользователь
    И пользователь пропустил онбординг
    Тогда user ID в IndexedDB не nil

  Сценарий: Два браузера на одном аккаунте имеют одну запись
    Допустим новый пользователь
    И пользователь пропустил онбординг
    Когда второй браузер входит в тот же аккаунт
    Тогда на сервере одна запись пользователя

  # Regression guard for the PR #303 incident: a wire-format change that trips
  # a server-side invariant (then: CHECK(json_valid(knowledge_set))) broke
  # every save_sync for existing accounts. This roundtrip verifies that data
  # mutated on a checkpoint path (toggle_favorite) reaches the remote and is
  # read back after a fresh login on a clean client. See ADR-034.
  Сценарий: Данные аккаунта сохраняются после повторного входа
    Допустим новый пользователь
    И пользователь пропустил онбординг
    Допустим у пользователя есть добавленное слово
    Когда отмечает первую карточку избранной и дожидается сохранения
    Тогда первая карточка отмечена избранной
    Допустим пользователь вышел из аккаунта
    Когда пользователь снова входит в тот же аккаунт
    Когда пользователь открывает страницу слов
    Тогда первая карточка отмечена избранной
