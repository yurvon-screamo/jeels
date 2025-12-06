# Duolingo integration

Profile request:

```bash
curl 'https://www.duolingo.com/2023-05-23/users/421382163?email,fromLanguage,learningLanguage,googleId,currentCourse,username&_=1765021324980'  -H 'content-type: application/json; charset=UTF-8' -H 'authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJleHAiOjYzMDcyMDAwMDAsImlhdCI6MCwic3ViIjo0MjEzODIxNjN9.vWr1CsZZX7QRN7lwzMpMnaFrfScc3hVYtONw61GlSqE'
```

Response:

```json
{
    "id": 421382163,
    "email": "yurvon@ya.ru",
    "username": "yurvon-screamo",
    "fromLanguage": "en",
    "learningLanguage": "ja",
    "googleId": "100797768697450716536",
    "currentCourse": {
        "skills": [
            [
                {
                    "finishedLessons": 3,
                    "id": "e1fafdce94f72b6efc4c4f825bc87b67"
                }
            ],
            [
                {
                    "finishedLessons": 5,
                    "id": "23980266cd20faed612dd6749718c26e"
                }
            ]
        ]
    }
}
```

After that, we make a request, where progressedSkills is taken from currentCourse.skills - finishedLevels is always 1, finishedSessions is taken from `finishedLessons + 1`.

Repeat with 1000 limit

curl -X POST 'https://www.duolingo.com/2017-06-30/users/421382163/courses/ja/en/learned-lexemes?limit=1000&sortBy=LEARNED_DATE&startIndex=0' -H 'content-type: application/json; charset=UTF-8' -H 'authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJleHAiOjYzMDcyMDAwMDAsImlhdCI6MCwic3ViIjo0MjEzODIxNjN9.vWr1CsZZX7QRN7lwzMpMnaFrfScc3hVYtONw61GlSqE'  --data-raw '{"progressedSkills":[{"finishedLevels":1,"finishedSessions":4,"skillId":{"id":"e1fafdce94f72b6efc4c4f825bc87b67"}},{"finishedLevels":1,"finishedSessions":4,"skillId":{"id":"23980266cd20faed612dd6749718c26e"}}]}'

Response format:

```json
{
    "learnedLexemes": [
        {
            "text": "トースト",
            "translations": [
                "toast"
            ]
        },
        {
            "text": "コーヒーメーカー",
            "translations": [
                "coffee maker"
            ]
        }
    ],
    "pagination": {
        "totalLexemes": 2,
        "requestedPageSize": 1000,
        "pageSize": 2,
        "previousStartIndex": null,
        "nextStartIndex": null
    }
}
```
