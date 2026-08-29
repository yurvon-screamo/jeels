use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::Deserialize;
use sha2::{Digest, Sha256};

#[derive(Debug, Deserialize)]
pub struct JwtClaims {
    pub sub: String,
    pub email: Option<String>,
    exp: Option<i64>,
    iss: Option<String>,
}

impl JwtClaims {
    pub fn expires_at(&self, fallback: u64) -> u64 {
        self.exp
            .and_then(|e| u64::try_from(e).ok())
            .unwrap_or(fallback)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OAuthProvider {
    Google,
    Yandex,
    Apple,
}

impl OAuthProvider {
    pub fn as_str(&self) -> &'static str {
        match self {
            OAuthProvider::Google => "google",
            OAuthProvider::Yandex => "yandex",
            // TrailBase provider id; configured with the SIWA Services ID
            // `net.uwuwu.origa.web` (see ADR-033 §SIWA secrets).
            OAuthProvider::Apple => "apple",
        }
    }
}

pub fn decode_jwt_claims(token: &str) -> Result<JwtClaims, String> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err("Invalid JWT format".to_string());
    }

    let payload = parts[1];
    let decoded = URL_SAFE_NO_PAD
        .decode(payload)
        .map_err(|e| format!("Failed to decode JWT payload: {}", e))?;
    let json_str =
        String::from_utf8(decoded).map_err(|e| format!("Invalid UTF-8 in JWT payload: {}", e))?;

    let claims: JwtClaims = serde_json::from_str(&json_str)
        .map_err(|e| format!("Failed to parse JWT claims: {}", e))?;

    if let Some(exp) = claims.exp {
        let now = chrono::Utc::now().timestamp();
        if now >= exp {
            return Err("Token expired".to_string());
        }
    }

    // Issuer validation is optional: TrailBase may not include iss in JWT
    // for some auth flows (OIDC). When iss is present and not from trusted
    // sources — log warning, but don't block, since all requests go to a known server.
    if let Some(ref iss) = claims.iss {
        let is_trusted = iss == "trailbase" || iss == env!("TRAILBASE_URL");
        if !is_trusted {
            tracing::warn!("Untrusted JWT issuer: {}", iss);
        }
    }

    Ok(claims)
}

pub fn generate_pkce_verifier() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~";
    let mut rng = rand::rng();
    (0..64)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

pub fn generate_pkce_challenge(verifier: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let result = hasher.finalize();
    URL_SAFE_NO_PAD.encode(result)
}

pub fn urlencoding_decode(s: &str) -> String {
    urlencoding::decode(s)
        .map(|cow| cow.into_owned())
        .unwrap_or_else(|_| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_jwt(payload: &str) -> String {
        let header = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(r#"{"alg":"HS256","typ":"JWT"}"#);
        let payload_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(payload);
        format!("{}.{}.signature", header, payload_b64)
    }

    #[test]
    fn decode_jwt_extracts_sub_from_valid_token() {
        let token = make_jwt(r#"{"sub":"user123","email":"test@test.com","exp":9999999999}"#);

        let claims = decode_jwt_claims(&token).unwrap();

        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.email.as_deref(), Some("test@test.com"));
    }

    #[test]
    fn decode_jwt_rejects_expired_token() {
        let token = make_jwt(r#"{"sub":"user123","exp":1}"#);

        let result = decode_jwt_claims(&token);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("expired"));
    }

    #[test]
    fn decode_jwt_rejects_invalid_format() {
        assert!(decode_jwt_claims("invalid").is_err());
        assert!(decode_jwt_claims("only.two").is_err());
        assert!(decode_jwt_claims("").is_err());
    }

    #[test]
    fn decode_jwt_rejects_malformed_payload() {
        let token = format!(
            "{}.{}.sig",
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(r#"{"alg":"HS256"}"#),
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode("not json")
        );

        assert!(decode_jwt_claims(&token).is_err());
    }

    #[test]
    fn pkce_verifier_has_required_length() {
        let verifier = generate_pkce_verifier();

        assert_eq!(verifier.len(), 64);
    }

    #[test]
    fn pkce_challenge_is_deterministic_for_same_verifier() {
        let verifier = "test_verifier_123";

        let challenge1 = generate_pkce_challenge(verifier);
        let challenge2 = generate_pkce_challenge(verifier);

        assert_eq!(challenge1, challenge2);
    }

    #[test]
    fn pkce_challenge_differs_for_different_verifiers() {
        let challenge1 = generate_pkce_challenge("verifier_a");
        let challenge2 = generate_pkce_challenge("verifier_b");

        assert_ne!(challenge1, challenge2);
    }

    #[test]
    fn urlencoding_decode_handles_percent_encoded_input() {
        assert_eq!(urlencoding_decode("hello%20world"), "hello world");
        assert_eq!(urlencoding_decode("test%40example.com"), "test@example.com");
    }

    #[test]
    fn urlencoding_decode_returns_original_on_invalid_input() {
        assert_eq!(urlencoding_decode("%E0%A4%A"), "%E0%A4%A");
    }

    #[test]
    fn oauth_provider_as_str_returns_correct_values() {
        assert_eq!(OAuthProvider::Google.as_str(), "google");
        assert_eq!(OAuthProvider::Yandex.as_str(), "yandex");
        // Must match the TrailBase provider id configured with the SIWA
        // Services ID (ADR-033) — a mismatch would 404 the login redirect.
        assert_eq!(OAuthProvider::Apple.as_str(), "apple");
    }

    #[test]
    fn jwt_claims_expires_at_returns_exp_when_present() {
        let token = make_jwt(r#"{"sub":"u","exp":9999999999}"#);

        let claims = decode_jwt_claims(&token).unwrap();

        assert_eq!(claims.expires_at(0), 9999999999);
    }

    #[test]
    fn jwt_claims_expires_at_returns_fallback_when_absent() {
        let token = make_jwt(r#"{"sub":"u"}"#);

        let claims = decode_jwt_claims(&token).unwrap();

        assert_eq!(claims.expires_at(42), 42);
    }
}
