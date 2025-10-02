use rand::{Rng, seq::SliceRandom};

use crate::AppConfig;
use anyhow::Result;

pub struct PexelsVideo {
    pub content: Vec<u8>,
}

pub async fn pick_video_from_pexels(
    config: &AppConfig,
    min_duration: usize,
) -> Result<Vec<PexelsVideo>> {
    let mut videos = Vec::new();

    let mut keywords = config.pexels_keywords.clone();
    keywords.shuffle(&mut rand::rng());

    for keyword in &keywords {
        let random_people_count: u32 = rand::rng().random_range(0..=2);
        let people_count = match random_people_count {
            0 => "0",
            1 => "1",
            _ => "3_plus",
        };

        let response = reqwest::Client::new()
            .get(format!(
                "https://api.pexels.com/videos/search?query={}&orientation=portrait&min_duration={}&people_count={}&per_page={}",
                keyword, min_duration, people_count, config.pexels_per_page
            ))
            .header("Authorization", config.pexels_api_key.clone())
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        let mut video_links = response
            .get("videos")
            .and_then(|v| v.as_array())
            .unwrap_or(&vec![])
            .into_iter()
            .filter_map(|v| {
                v.get("video_files")
                    .and_then(|v| v.as_array().and_then(|a| a.into_iter().skip(1).next()))
                    .and_then(|v| {
                        v.get("link")
                            .and_then(|v| v.as_str().map(|s| s.to_string()))
                    })
            })
            .collect::<Vec<_>>();

        video_links.shuffle(&mut rand::rng());

        for link in video_links {
            let response = reqwest::Client::new()
                .get(link)
                .send()
                .await?
                .bytes()
                .await?;

            videos.push(PexelsVideo {
                content: response.to_vec(),
            });

            if videos.len() >= config.pexels_total {
                return Ok(videos);
            }
        }
    }

    Err(anyhow::anyhow!("No videos found for required total"))
}
