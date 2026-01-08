use crate::application::{DuolingoClient, DuolingoWord};
use crate::domain::KeikakuError;
use async_trait::async_trait;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

#[derive(Debug, Deserialize)]
struct DuolingoUserProfileResponse {
    #[serde(rename = "learningLanguage")]
    learning_language: String,
    #[serde(rename = "fromLanguage")]
    from_language: String,
    #[serde(rename = "currentCourse")]
    current_course: Option<DuolingoCurrentCourse>,
}

#[derive(Debug, Deserialize)]
struct DuolingoCurrentCourse {
    skills: Vec<Vec<DuolingoSkillDto>>,
}

#[derive(Debug, Deserialize)]
struct DuolingoSkillDto {
    id: String,
    #[serde(rename = "finishedLessons", default)]
    finished_lessons: u32,
    #[serde(rename = "finishedLevels", default)]
    finished_levels: u32,
}

#[derive(Debug, Serialize)]
struct LearnedLexemesRequest {
    #[serde(rename = "progressedSkills")]
    progressed_skills: Vec<DuolingoProgressedSkillPayload>,
}

#[derive(Debug, Clone, Serialize)]
struct DuolingoProgressedSkillPayload {
    #[serde(rename = "finishedLevels")]
    finished_levels: u32,
    #[serde(rename = "finishedSessions")]
    finished_sessions: u32,
    #[serde(rename = "skillId")]
    skill_id: DuolingoSkillId,
}

#[derive(Debug, Clone, Serialize)]
struct DuolingoSkillId {
    id: String,
}

#[derive(Debug, Deserialize)]
struct LearnedLexemesResponse {
    #[serde(rename = "learnedLexemes")]
    learned_lexemes: Vec<DuolingoLexemeDto>,
    #[serde(rename = "pagination")]
    pagination: DuolingoPagination,
}

#[derive(Debug, Deserialize)]
struct DuolingoLexemeDto {
    #[serde(rename = "text")]
    text: String,
    #[serde(rename = "translations")]
    translations: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct DuolingoPagination {
    #[serde(rename = "nextStartIndex")]
    next_start_index: Option<u32>,
}

pub struct HttpDuolingoClient {
    client: reqwest::Client,
}

impl Default for HttpDuolingoClient {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpDuolingoClient {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }
}

#[async_trait]
impl DuolingoClient for HttpDuolingoClient {
    async fn get_words(&self, jwt_token: &str) -> Result<Vec<DuolingoWord>, KeikakuError> {
        let user_id = extract_user_id_from_jwt(jwt_token)?;

        let profile = get_user_profile(&self.client, &user_id, jwt_token).await?;

        let empty_skills: Vec<Vec<DuolingoSkillDto>> = Vec::new();
        let skills = profile
            .current_course
            .as_ref()
            .map(|course| &course.skills)
            .unwrap_or(&empty_skills);

        let progressed_skills = build_progressed_skills(skills);

        let words = get_learned_lexemes(
            &self.client,
            &user_id,
            &profile.learning_language,
            &profile.from_language,
            jwt_token,
            &progressed_skills,
        )
        .await?;

        Ok(words)
    }
}

fn extract_user_id_from_jwt(jwt_token: &str) -> Result<String, KeikakuError> {
    let parts: Vec<&str> = jwt_token.split('.').collect();
    if parts.len() != 3 {
        return Err(KeikakuError::RepositoryError {
            reason: "Invalid JWT token format".to_string(),
        });
    }

    let payload = parts[1];
    let decoded = URL_SAFE_NO_PAD
        .decode(payload)
        .map_err(|e| KeikakuError::RepositoryError {
            reason: format!("Failed to decode JWT payload: {}", e),
        })?;

    let json: Value =
        serde_json::from_slice(&decoded).map_err(|e| KeikakuError::RepositoryError {
            reason: format!("Failed to parse JWT payload: {}", e),
        })?;

    let sub = json
        .get("sub")
        .ok_or_else(|| KeikakuError::RepositoryError {
            reason: format!("JWT token does not contain 'sub' field: {}", json),
        })?;

    let sub_str = sub
        .as_str()
        .map(|s| s.to_string())
        .or_else(|| sub.as_u64().map(|n| n.to_string()))
        .or_else(|| sub.as_i64().map(|n| n.to_string()))
        .ok_or_else(|| KeikakuError::RepositoryError {
            reason: format!("JWT token 'sub' field is not a string or number: {}", json),
        })?;

    Ok(sub_str)
}

async fn get_user_profile(
    client: &reqwest::Client,
    user_id: &str,
    jwt_token: &str,
) -> Result<DuolingoUserProfileResponse, KeikakuError> {
    let url = format!(
        "https://www.duolingo.com/2023-05-23/users/{}?email,fromLanguage,learningLanguage,googleId,currentCourse,username&_={}",
        user_id,
        chrono::Utc::now().timestamp_millis()
    );

    let response = client
        .get(&url)
        .header("authorization", format!("Bearer {}", jwt_token))
        .header("User-Agent", "curl/8.16.0")
        .header("Accept", "*/*")
        .send()
        .await
        .map_err(|e| KeikakuError::RepositoryError {
            reason: format!("Failed to fetch Duolingo profile: {}", e),
        })?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(KeikakuError::RepositoryError {
            reason: format!("Duolingo API error: {} {}", status, text),
        });
    }

    let profile: DuolingoUserProfileResponse =
        response
            .json()
            .await
            .map_err(|e| KeikakuError::RepositoryError {
                reason: format!("Failed to parse Duolingo profile: {}", e),
            })?;

    Ok(profile)
}

fn build_progressed_skills(
    skills: &[Vec<DuolingoSkillDto>],
) -> Vec<DuolingoProgressedSkillPayload> {
    let mut progressed_skills = Vec::new();

    for section in skills {
        for skill in section {
            if skill.finished_levels > 0 {
                progressed_skills.push(DuolingoProgressedSkillPayload {
                    finished_levels: 1,
                    finished_sessions: skill.finished_lessons + 1,
                    skill_id: DuolingoSkillId {
                        id: skill.id.clone(),
                    },
                });
            }
        }
    }

    progressed_skills
}

async fn get_learned_lexemes(
    client: &reqwest::Client,
    user_id: &str,
    learning_lang: &str,
    from_lang: &str,
    jwt_token: &str,
    progressed_skills: &[DuolingoProgressedSkillPayload],
) -> Result<Vec<DuolingoWord>, KeikakuError> {
    let base_url = format!(
        "https://www.duolingo.com/2017-06-30/users/{}/courses/{}/{}/learned-lexemes",
        user_id, learning_lang, from_lang
    );

    let mut all_words = Vec::new();
    let mut start_index = 0u32;
    let limit = 1000u32;

    loop {
        let payload = LearnedLexemesRequest {
            progressed_skills: progressed_skills.to_vec(),
        };

        let url = format!(
            "{}?limit={}&sortBy=LEARNED_DATE&startIndex={}",
            base_url, limit, start_index
        );

        let response: reqwest::Response = client
            .post(&url)
            .header("authorization", format!("Bearer {}", jwt_token))
            .header("content-type", "application/json; charset=UTF-8")
            .header("User-Agent", "curl/8.16.0")
            .header("Accept", "*/*")
            .json(&payload)
            .send()
            .await
            .map_err(|e| KeikakuError::RepositoryError {
                reason: format!("Failed to fetch Duolingo lexemes: {}", e),
            })?;

        let status = response.status();
        let data: LearnedLexemesResponse = if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(KeikakuError::RepositoryError {
                reason: format!("Duolingo API error: {} {}", status, text),
            });
        } else {
            response
                .json()
                .await
                .map_err(|e| KeikakuError::RepositoryError {
                    reason: format!("Failed to parse Duolingo lexemes: {}", e),
                })?
        };

        all_words.extend(data.learned_lexemes.into_iter().map(|lexeme| DuolingoWord {
            text: lexeme.text,
            translations: lexeme.translations,
        }));

        match data.pagination.next_start_index {
            Some(next_index) => {
                start_index = next_index;
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
            None => break,
        }
    }

    Ok(all_words)
}
