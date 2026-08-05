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
  # every save_sync for existing accounts. This scenario drives a real
  # checkpoint save (toggle_favorite) and asserts both that the save_sync
  # request returned 2xx AND that the remote row on disk reflects the write —
  # catching a server-side invariant rejection (HTTP 500) without depending
  # on a flaky second UI login. See ADR-034.
  Сценарий: Данные аккаунта сохраняются на сервере
    Допустим новый пользователь
    И пользователь пропустил онбординг
    Допустим у пользователя есть добавленное слово
    Когда отмечает первую карточку избранной и дожидается сохранения
    Тогда первая карточка отмечена избранной
    И запись на сервере содержит обновлённый knowledge_set

  # Regression guard for the MemoryHistory denormalization (v12): the
  # reviews array was replaced with scalar counters. This scenario drives
  # a real lesson (rate → save_sync) and asserts the remote row reflects
  # the write — catching a serialization regression where the denormalized
  # format is rejected or lost.
  Сценарий: Memory state сохраняется на сервере после урока
    Допустим новый пользователь
    И пользователь пропустил онбординг
    И у пользователя есть добавленное слово
    Когда пользователь начинает урок
    И оценивает каждую карточку как Good
    И дожидается сохранения данных урока
    Тогда запись на сервере содержит обновлённый knowledge_set
