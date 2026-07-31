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
  # every save_sync for existing accounts. This scenario verifies that data
  # mutated on a checkpoint path (toggle_favorite) reaches the remote and is
  # read back from a SECOND, pristine browser context — the real cross-device
  # sync roundtrip. See ADR-034.
  Сценарий: Данные аккаунта сохраняются и видны с другого устройства
    Допустим новый пользователь
    И пользователь пропустил онбординг
    Допустим у пользователя есть добавленное слово
    Когда отмечает первую карточку избранной и дожидается сохранения
    Тогда первая карточка отмечена избранной
    И второй браузер видит эту карточку избранной
