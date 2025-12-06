import json
import time

import requests

USER_ID = "421382163"
TOKEN = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJleHAiOjYzMDcyMDAwMDAsImlhdCI6MCwic3ViIjo0MjEzODIxNjN9.vWr1CsZZX7QRN7lwzMpMnaFrfScc3hVYtONw61GlSqE"

HEADERS = {
    "authorization": f"Bearer {TOKEN}",
    "User-Agent": "curl/8.16.0",
    "Accept": "*/*",
}


def get_user_profile():
    url = f"https://www.duolingo.com/2023-05-23/users/{USER_ID}?email,fromLanguage,learningLanguage,googleId,currentCourse,username&_=1765021324980"

    print(f"[*] Запрашиваем профиль пользователя {USER_ID}...")
    print(f"[DEBUG] URL: {url}")
    print(f"[DEBUG] Headers: {HEADERS}")

    response = requests.get(url, headers=HEADERS)

    print(f"[DEBUG] Status code: {response.status_code}")
    print(f"[DEBUG] Response headers: {dict(response.headers)}")

    if response.status_code != 200:
        error_text = response.text
        print(f"[!] Error getting user profile: {response.status_code} {error_text}")
        return None

    return response.json()


def build_progressed_skills(skills_nested_list):
    progressed_skills = []

    for section in skills_nested_list:
        for skill in section:
            finished_levels = skill.get("finishedLevels", 0)
            if finished_levels > 0:
                # finishedLevels eq 1
                # finishedSessions eq finishedLessons + 1
                finished_lessons = skill.get("finishedLessons", 0)
                skill_obj = {
                    "finishedLevels": 1,
                    "finishedSessions": finished_lessons + 1,
                    "skillId": {"id": skill.get("id")},
                }
                progressed_skills.append(skill_obj)

    return progressed_skills


def fetch_all_lexemes(learning_lang, from_lang, progressed_skills):
    all_lexemes = []
    base_url = f"https://www.duolingo.com/2017-06-30/users/{USER_ID}/courses/{learning_lang}/{from_lang}/learned-lexemes"

    limit = 1000
    start_index = 0

    post_headers = {
        **HEADERS,
        "content-type": "application/json; charset=UTF-8",
    }

    while True:
        print(f"[*] Загрузка слов с {start_index}...")

        params = {"limit": limit, "sortBy": "LEARNED_DATE", "startIndex": start_index}
        payload = {"progressedSkills": progressed_skills}

        response = requests.post(
            base_url, headers=post_headers, params=params, json=payload
        )

        if response.status_code != 200:
            print(f"[!] Ошибка загрузки слов: {response.status_code}")
            print(response.text)
            break

        data = response.json()
        lexemes = data.get("learnedLexemes", [])
        all_lexemes.extend(
            {"text": lexeme.get("text"), "translations": lexeme.get("translations")}
            for lexeme in lexemes
        )

        print(f"    Loaded {len(lexemes)} words.")
        pagination = data.get("pagination", {})
        print(f"[DEBUG] Pagination: {pagination}")
        next_index = pagination.get("nextStartIndex")
        if next_index is None:
            break
        start_index = next_index
        time.sleep(1)

    return all_lexemes


def main():
    profile = get_user_profile()
    if not profile:
        return

    learning_lang = profile.get("learningLanguage")  # as 'ja'
    from_lang = profile.get("fromLanguage")  # as'en'
    current_course = profile.get("currentCourse", {})
    skills = current_course.get("skills", [])

    if not skills:
        print("[!] Не найдены навыки (skills) в текущем курсе.")
        return

    print(f"[*] Курс: {learning_lang} <- {from_lang}")

    progressed_skills_payload = build_progressed_skills(skills)
    print(f"[*] Подготовлено {len(progressed_skills_payload)} навыков для запроса.")

    words = fetch_all_lexemes(learning_lang, from_lang, progressed_skills_payload)

    filename = f"duolingo_words_{learning_lang}.json"
    with open(filename, "w", encoding="utf-8") as f:
        json.dump(words, f, ensure_ascii=False, indent=4)

    print("---")
    print(f"[SUCCESS] Всего сохранено слов: {len(words)}")
    print(f"Файл сохранен как: {filename}")


if __name__ == "__main__":
    main()
