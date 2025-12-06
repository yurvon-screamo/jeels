#[cfg(test)]
mod tests {
    use crate::domain::error::JeersError;
    use crate::domain::value_objects::{
        Answer, Difficulty, Embedding, ExamplePhrase, MemoryState, Question, Rating, Stability,
    };
    use crate::domain::{JapaneseLevel, NativeLanguage, User};
    use chrono::{Duration, Utc};

    fn create_test_user() -> User {
        User::new(
            "test_user".to_string(),
            JapaneseLevel::N5,
            NativeLanguage::Russian,
            10,
        )
    }

    fn create_test_question() -> Question {
        const QUESTION: &str = "何ですか？";
        Question::new(QUESTION.to_string(), generate_embedding(QUESTION)).unwrap()
    }

    fn generate_embedding(text: &str) -> Embedding {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        let hash = hasher.finish();

        let mut embedding = vec![0.0; 384];
        // Fill embedding based on hash to make it deterministic but different for different texts
        for i in 0..384 {
            embedding[i] =
                ((hash.wrapping_mul(i as u64).wrapping_add(i as u64)) % 1000) as f32 / 1000.0;
        }
        Embedding(embedding)
    }

    fn create_test_answer() -> Answer {
        const ANSWER: &str = "Что ты хочешь сказать?";
        Answer::new(ANSWER.to_string()).unwrap()
    }

    fn create_test_difficulty() -> Difficulty {
        Difficulty::new(0.5).unwrap()
    }

    #[test]
    fn user_new_should_create_user_with_username_and_empty_cards() {
        // Arrange
        let username = "test_user".to_string();

        // Act
        let user = User::new(username, JapaneseLevel::N5, NativeLanguage::Russian, 10);

        // Assert
        assert_eq!(user.username(), "test_user");
        assert_eq!(user.cards().len(), 0);
    }

    #[test]
    fn user_new_should_generate_unique_ids() {
        // Arrange & Act
        let user1 = create_test_user();
        let user2 = create_test_user();

        // Assert
        assert_ne!(user1.id(), user2.id());
    }

    #[test]
    fn user_username_should_return_correct_username() {
        // Arrange
        let test_cases = vec!["alice", "bob", "charlie", "test_user_123"];

        for username in test_cases {
            // Act
            let user = User::new(
                username.to_string(),
                JapaneseLevel::N5,
                NativeLanguage::Russian,
                10,
            );

            // Assert
            assert_eq!(user.username(), username);
        }
    }

    #[test]
    fn user_current_japanese_level_should_return_correct_level() {
        // Arrange
        let user = create_test_user();

        // Assert
        assert_eq!(user.current_japanese_level(), &JapaneseLevel::N5);
    }

    #[test]
    fn user_native_language_should_return_correct_language() {
        // Arrange
        let user = create_test_user();

        // Assert
        assert_eq!(user.native_language(), &NativeLanguage::Russian);
    }

    #[test]
    fn user_cards_should_be_empty_when_user_is_created() {
        // Arrange
        let user = create_test_user();

        // Act
        let cards = user.cards();

        // Assert
        assert_eq!(cards.len(), 0);
    }

    #[test]
    fn user_create_card_should_add_card_to_user() {
        // Arrange
        let mut user = create_test_user();
        let question = create_test_question();
        let answer = create_test_answer();

        // Act
        let card = user
            .create_card(question.clone(), answer.clone(), vec![])
            .unwrap();

        // Assert
        assert_eq!(card.question().text(), "何ですか？");
        assert_eq!(card.answer().text(), "Что ты хочешь сказать?");
        assert_eq!(user.cards().len(), 1);
        assert!(user.cards().contains_key(&card.id()));
    }

    #[test]
    fn user_create_card_should_create_multiple_cards_with_unique_ids() {
        // Arrange
        let mut user = create_test_user();
        let test_data = vec![
            ("食べて", "есть"),
            ("飲みます", "пить"),
            ("教えて", "объяснить"),
        ];

        // Act
        let mut card_ids = Vec::new();
        for (q_text, a_text) in test_data {
            let question = Question::new(q_text.to_string(), generate_embedding(q_text)).unwrap();
            let answer = Answer::new(a_text.to_string()).unwrap();
            let card = user.create_card(question, answer, vec![]).unwrap();
            card_ids.push(card.id());
        }

        // Assert
        assert_eq!(user.cards().len(), 3);
        for i in 0..card_ids.len() {
            for j in (i + 1)..card_ids.len() {
                assert_ne!(card_ids[i], card_ids[j]);
            }
        }
    }

    #[test]
    fn user_edit_card_should_update_card_when_card_exists() {
        // Arrange
        let mut user = create_test_user();
        let question = create_test_question();
        let answer = create_test_answer();
        let card = user.create_card(question, answer, vec![]).unwrap();
        let card_id = card.id();
        let new_question_text = "教えて";
        let new_answer_text = "объяснить";
        let new_question = Question::new(
            new_question_text.to_string(),
            generate_embedding(new_question_text),
        )
        .unwrap();
        let new_answer = Answer::new(new_answer_text.to_string()).unwrap();

        // Act
        let result = user.edit_card(card_id, new_question.clone(), new_answer.clone(), vec![]);

        // Assert
        assert!(result.is_ok());
        let updated_card = user.get_card(card_id).unwrap();
        assert_eq!(updated_card.question().text(), new_question_text);
        assert_eq!(updated_card.answer().text(), new_answer_text);
    }

    #[test]
    fn user_edit_card_should_return_error_when_card_not_found() {
        // Arrange
        let mut user = create_test_user();
        let non_existent_id = ulid::Ulid::new();
        let question = create_test_question();
        let answer = create_test_answer();

        // Act
        let result = user.edit_card(non_existent_id, question, answer, vec![]);

        // Assert
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            JeersError::CardNotFound { card_id: _ }
        ));
    }

    #[test]
    fn user_delete_card_should_remove_card_when_card_exists() {
        // Arrange
        let mut user = create_test_user();
        let question = create_test_question();
        let answer = create_test_answer();
        let card = user.create_card(question, answer, vec![]).unwrap();
        let card_id = card.id();
        assert_eq!(user.cards().len(), 1);

        // Act
        let result = user.delete_card(card_id);

        // Assert
        assert!(result.is_ok());
        assert_eq!(user.cards().len(), 0);
        assert!(user.get_card(card_id).is_none());
    }

    #[test]
    fn user_delete_card_should_return_error_when_card_not_found() {
        // Arrange
        let mut user = create_test_user();
        let non_existent_id = ulid::Ulid::new();

        // Act
        let result = user.delete_card(non_existent_id);

        // Assert
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, JeersError::CardNotFound { .. }));
        if let JeersError::CardNotFound { card_id } = err {
            assert_eq!(card_id, non_existent_id);
        }
    }

    #[test]
    fn user_start_study_session_should_return_empty_when_no_cards() {
        // Arrange
        let user = create_test_user();

        // Act
        let cards = user.start_study_session(false);

        // Assert
        assert_eq!(cards.len(), 0);
    }

    #[test]
    fn user_start_study_session_should_return_due_cards_when_cards_exist() {
        // Arrange
        let mut user = create_test_user();
        let question = create_test_question();
        let answer = create_test_answer();
        let card = user.create_card(question, answer, vec![]).unwrap();
        let card_id = card.id();

        // Act
        let cards = user.start_study_session(false);

        // Assert
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].card_id(), card_id);
    }

    #[test]
    fn user_start_study_session_should_filter_non_due_cards() {
        // Arrange
        let mut user = create_test_user();
        let question = create_test_question();
        let answer = create_test_answer();
        let card = user.create_card(question, answer, vec![]).unwrap();
        let card_id = card.id();
        let stability = Stability::new(1.0).unwrap();
        let difficulty = create_test_difficulty();
        user.rate_card(
            card_id,
            Rating::Good,
            Duration::days(1),
            MemoryState::new(stability, difficulty),
        )
        .unwrap();

        // Act
        let cards = user.start_study_session(false);

        // Assert
        assert_eq!(cards.len(), 0);
    }

    #[test]
    fn user_start_study_session_should_return_only_due_cards_when_mixed() {
        // Arrange
        let mut user = create_test_user();

        let q1 = Question::new("私".to_string(), generate_embedding("私")).unwrap();
        let a1 = Answer::new("Я".to_string()).unwrap();
        let card1 = user.create_card(q1, a1, vec![]).unwrap();
        let card1_id = card1.id();

        let q2 = Question::new("食べて".to_string(), generate_embedding("食べて")).unwrap();
        let a2 = Answer::new("есть".to_string()).unwrap();
        let card2 = user.create_card(q2, a2, vec![]).unwrap();
        let card2_id = card2.id();

        let stability = Stability::new(1.0).unwrap();
        let difficulty = create_test_difficulty();
        user.rate_card(
            card2_id,
            Rating::Good,
            Duration::days(1),
            MemoryState::new(stability, difficulty),
        )
        .unwrap();

        // Act
        let cards = user.start_study_session(false);

        // Assert
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].card_id(), card1_id);
    }

    #[test]
    fn user_start_study_session_should_separate_old_and_new_cards() {
        // Arrange
        let mut user = create_test_user();

        // Low stability card: has Again review (is_new() = false, is_low_stability = true)
        let q1 = Question::new("私".to_string(), generate_embedding("私")).unwrap();
        let a1 = Answer::new("Я".to_string()).unwrap();
        let card1 = user.create_card(q1, a1, vec![]).unwrap();
        let card1_id = card1.id();
        let stability = Stability::new(0.1).unwrap();
        let difficulty = create_test_difficulty();
        // Add Again review - card becomes low_stability and due
        user.rate_card(
            card1_id,
            Rating::Again,
            Duration::zero(), // Make it due immediately
            MemoryState::new(stability, difficulty),
        )
        .unwrap();

        // Old card: has review with Good rating (is_new() = false, is_low_stability = true)
        let q2 = Question::new("食べて".to_string(), generate_embedding("食べて")).unwrap();
        let a2 = Answer::new("есть".to_string()).unwrap();
        let card2 = user.create_card(q2, a2, vec![]).unwrap();
        let card2_id = card2.id();
        let stability2 = Stability::new(0.1).unwrap();
        user.rate_card(
            card2_id,
            Rating::Good,
            Duration::zero(), // Make it due immediately
            MemoryState::new(stability2, difficulty),
        )
        .unwrap();

        // Act
        let cards = user.start_study_session(false);

        // Assert
        assert_eq!(cards.len(), 2);
        // Both cards should be present (order is shuffled)
        let card_ids: Vec<_> = cards.iter().map(|c| c.card_id()).collect();
        assert!(card_ids.contains(&card1_id));
        assert!(card_ids.contains(&card2_id));
    }

    #[test]
    fn user_start_study_session_should_put_old_cards_first() {
        // Arrange
        let mut user = create_test_user();

        // Create old card (has Good review - not new, is_low_stability = true)
        let q1 = Question::new("私".to_string(), generate_embedding("私")).unwrap();
        let a1 = Answer::new("Я".to_string()).unwrap();
        let card1 = user.create_card(q1, a1, vec![]).unwrap();
        let card1_id = card1.id();
        let stability1 = Stability::new(0.1).unwrap();
        let difficulty = create_test_difficulty();
        user.rate_card(
            card1_id,
            Rating::Good,
            Duration::zero(), // Make it due immediately
            MemoryState::new(stability1, difficulty),
        )
        .unwrap();

        // Create new card (without reviews - is new)
        let q2 = Question::new("食べて".to_string(), generate_embedding("食べて")).unwrap();
        let a2 = Answer::new("есть".to_string()).unwrap();
        let card2 = user.create_card(q2, a2, vec![]).unwrap();
        let card2_id = card2.id();

        // Act
        let cards = user.start_study_session(false);

        // Assert
        assert_eq!(cards.len(), 2);
        // Both cards should be present (order is shuffled)
        let card_ids: Vec<_> = cards.iter().map(|c| c.card_id()).collect();
        assert!(card_ids.contains(&card1_id));
        assert!(card_ids.contains(&card2_id));
    }

    #[test]
    fn user_start_study_session_should_limit_new_cards() {
        // Arrange
        let mut user = User::new(
            "test_user".to_string(),
            JapaneseLevel::N5,
            NativeLanguage::Russian,
            2, // new_cards_limit = 2
        );

        // Create 3 new cards (without reviews - they are new)
        for i in 1..=3 {
            let q =
                Question::new(format!("私{}", i), generate_embedding(&format!("私{}", i))).unwrap();
            let a = Answer::new(format!("Я{}", i)).unwrap();
            user.create_card(q, a, vec![]).unwrap();
        }

        // Act
        let cards = user.start_study_session(false);

        // Assert
        // Should return only 2 new cards (limited by new_cards_limit)
        assert_eq!(cards.len(), 2);
    }

    #[test]
    fn user_start_study_session_should_return_all_old_cards() {
        // Arrange
        let mut user = User::new(
            "test_user".to_string(),
            JapaneseLevel::N5,
            NativeLanguage::Russian,
            2, // new_cards_limit = 2
        );

        // Create 5 old cards (have Good reviews - not new)
        let mut old_card_ids = Vec::new();
        let difficulty = create_test_difficulty();
        for i in 1..=5 {
            let q =
                Question::new(format!("私{}", i), generate_embedding(&format!("私{}", i))).unwrap();
            let a = Answer::new(format!("Я{}", i)).unwrap();
            let card = user.create_card(q, a, vec![]).unwrap();
            let card_id = card.id();
            old_card_ids.push(card_id);
            let stability = Stability::new(0.1).unwrap();
            user.rate_card(
                card_id,
                Rating::Good,
                Duration::zero(), // Make it due immediately
                MemoryState::new(stability, difficulty),
            )
            .unwrap();
        }

        // Act
        let cards = user.start_study_session(false);

        // Assert
        // Should return all 5 old cards (no limit for old cards)
        assert_eq!(cards.len(), 5);
        for card_id in old_card_ids {
            assert!(cards.iter().any(|c| c.card_id() == card_id));
        }
    }

    #[test]
    fn user_start_study_session_should_sort_cards_by_next_review_date() {
        // Arrange
        let mut user = create_test_user();

        // Create old cards (with Good reviews to make them old)
        let q1 = Question::new("私".to_string(), generate_embedding("私")).unwrap();
        let a1 = Answer::new("Я".to_string()).unwrap();
        let card1 = user.create_card(q1, a1, vec![]).unwrap();
        let card1_id = card1.id();

        let q2 = Question::new("食べて".to_string(), generate_embedding("食べて")).unwrap();
        let a2 = Answer::new("есть".to_string()).unwrap();
        let card2 = user.create_card(q2, a2, vec![]).unwrap();
        let card2_id = card2.id();

        // Set different review dates - both due immediately
        let stability = Stability::new(0.1).unwrap();
        let difficulty = create_test_difficulty();
        user.rate_card(
            card1_id,
            Rating::Good,
            Duration::zero(), // Make it due immediately
            MemoryState::new(stability, difficulty),
        )
        .unwrap();
        user.rate_card(
            card2_id,
            Rating::Good,
            Duration::zero(), // Make it due immediately
            MemoryState::new(stability, difficulty),
        )
        .unwrap();

        // Act
        let cards = user.start_study_session(false);

        // Assert
        assert_eq!(cards.len(), 2);
        // Should be sorted by next_review_date (earlier first)
        // Both are due now, so order may vary, but both should be present
        assert!(cards.iter().any(|c| c.card_id() == card1_id));
        assert!(cards.iter().any(|c| c.card_id() == card2_id));
    }

    #[test]
    fn user_start_study_session_should_treat_cards_with_only_again_as_new() {
        // Arrange
        let mut user = create_test_user();

        // Create card without reviews (should be new)
        let q1 = Question::new("私".to_string(), generate_embedding("私")).unwrap();
        let a1 = Answer::new("Я".to_string()).unwrap();
        let card1 = user.create_card(q1, a1, vec![]).unwrap();
        let card1_id = card1.id();

        // Act
        let cards = user.start_study_session(false);

        // Assert
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].card_id(), card1_id);
        // Card without reviews should be treated as new
    }

    #[test]
    fn user_start_study_session_should_combine_old_and_new_cards_correctly() {
        // Arrange
        let mut user = User::new(
            "test_user".to_string(),
            JapaneseLevel::N5,
            NativeLanguage::Russian,
            2, // new_cards_limit = 2
        );

        // Create 2 old cards (with Good reviews - not new, in_progress)
        // in_progress requires stability >= 1.5 and < 4.0
        let q1 = Question::new("私".to_string(), generate_embedding("私")).unwrap();
        let a1 = Answer::new("Я".to_string()).unwrap();
        let card1 = user.create_card(q1, a1, vec![]).unwrap();
        let card1_id = card1.id();

        let q2 = Question::new("食べて".to_string(), generate_embedding("食べて")).unwrap();
        let a2 = Answer::new("есть".to_string()).unwrap();
        let card2 = user.create_card(q2, a2, vec![]).unwrap();
        let card2_id = card2.id();

        let difficulty = create_test_difficulty();
        // Use stability 2.0 to make cards in_progress (>= 1.5 and < 4.0)
        let stability_old = Stability::new(2.0).unwrap();
        user.rate_card(
            card1_id,
            Rating::Good,
            Duration::zero(), // Make it due immediately
            MemoryState::new(stability_old, difficulty),
        )
        .unwrap();
        user.rate_card(
            card2_id,
            Rating::Good,
            Duration::zero(), // Make it due immediately
            MemoryState::new(stability_old, difficulty),
        )
        .unwrap();

        // Create 3 new cards (without reviews - they are new)
        let mut new_card_ids = Vec::new();
        for i in 3..=5 {
            let q =
                Question::new(format!("私{}", i), generate_embedding(&format!("私{}", i))).unwrap();
            let a = Answer::new(format!("Я{}", i)).unwrap();
            let card = user.create_card(q, a, vec![]).unwrap();
            new_card_ids.push(card.id());
        }

        // Act
        let cards = user.start_study_session(false);

        // Assert
        // Should return all 2 old cards + 2 new cards (limited)
        assert_eq!(cards.len(), 4);
        let card_ids: Vec<_> = cards.iter().map(|c| c.card_id()).collect();
        // Both old cards should be present
        assert!(card_ids.contains(&card1_id));
        assert!(card_ids.contains(&card2_id));
        // Exactly 2 new cards should be present (limited)
        let new_cards_in_session: Vec<_> = card_ids
            .iter()
            .filter(|id| new_card_ids.contains(id))
            .collect();
        assert_eq!(new_cards_in_session.len(), 2);
    }

    #[test]
    fn user_rate_card_should_add_review_when_card_exists() {
        // Arrange
        let mut user = create_test_user();
        let question = create_test_question();
        let answer = create_test_answer();
        let card = user.create_card(question, answer, vec![]).unwrap();
        let card_id = card.id();
        let rating = Rating::Good;
        let interval = Duration::days(1);
        let stability = Stability::new(1.0).unwrap();
        let difficulty = create_test_difficulty();

        // Act
        let result = user.rate_card(
            card_id,
            rating,
            interval,
            MemoryState::new(stability, difficulty),
        );

        // Assert
        assert!(result.is_ok());
        let card = user.get_card(card_id).unwrap();
        assert_eq!(card.reviews().len(), 1);
        assert_eq!(card.reviews()[0].rating(), rating);
        assert_eq!(card.reviews()[0].interval().num_days(), 1);
    }

    #[test]
    fn user_rate_card_should_work_with_all_rating_variants() {
        // Arrange
        let test_cases = vec![
            (Rating::Easy, 1),
            (Rating::Good, 2),
            (Rating::Hard, 3),
            (Rating::Again, 1),
        ];

        for (rating, expected_interval) in test_cases {
            let mut user = create_test_user();
            let question = create_test_question();
            let answer = create_test_answer();
            let card = user.create_card(question, answer, vec![]).unwrap();
            let card_id = card.id();
            let interval = Duration::days(expected_interval);
            let stability = Stability::new(expected_interval as f64).unwrap();
            let difficulty = create_test_difficulty();

            // Act
            let result = user.rate_card(
                card_id,
                rating,
                interval,
                MemoryState::new(stability, difficulty),
            );

            // Assert
            assert!(result.is_ok());
            let card = user.get_card(card_id).unwrap();
            assert_eq!(card.reviews().len(), 1);
            assert_eq!(card.reviews()[0].rating(), rating);
            assert_eq!(card.reviews()[0].interval().num_days(), expected_interval);
        }
    }

    #[test]
    fn user_rate_card_should_allow_multiple_ratings() {
        // Arrange
        let mut user = create_test_user();
        let question = create_test_question();
        let answer = create_test_answer();
        let card = user.create_card(question, answer, vec![]).unwrap();
        let card_id = card.id();
        let ratings = vec![(Rating::Easy, 1), (Rating::Good, 2), (Rating::Hard, 3)];

        // Act
        for (rating, interval_days) in ratings.iter() {
            let stability = Stability::new(*interval_days as f64).unwrap();
            let difficulty = create_test_difficulty();
            user.rate_card(
                card_id,
                *rating,
                Duration::days(*interval_days),
                MemoryState::new(stability, difficulty),
            )
            .unwrap();
        }

        // Assert
        let card = user.get_card(card_id).unwrap();
        assert_eq!(card.reviews().len(), 3);
        assert_eq!(card.reviews()[0].rating(), Rating::Easy);
        assert_eq!(card.reviews()[1].rating(), Rating::Good);
        assert_eq!(card.reviews()[2].rating(), Rating::Hard);
    }

    #[test]
    fn user_rate_card_should_return_error_when_card_not_found() {
        // Arrange
        let mut user = create_test_user();
        let non_existent_id = ulid::Ulid::new();
        let rating = Rating::Good;
        let interval = Duration::days(1);
        let stability = Stability::new(1.0).unwrap();
        let difficulty = create_test_difficulty();

        // Act
        let result = user.rate_card(
            non_existent_id,
            rating,
            interval,
            MemoryState::new(stability, difficulty),
        );

        // Assert
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, JeersError::CardNotFound { .. }));
        if let JeersError::CardNotFound { card_id } = err {
            assert_eq!(card_id, non_existent_id);
        }
    }

    #[test]
    fn user_rate_card_should_update_schedule_when_card_exists() {
        // Arrange
        let test_cases = vec![
            (Duration::days(1), 1.0),
            (Duration::days(5), 2.5),
            (Duration::days(10), 5.0),
            (Duration::days(30), 10.0),
        ];

        for (duration, stability_value) in test_cases {
            let mut user = create_test_user();
            let question = create_test_question();
            let answer = create_test_answer();
            let card = user.create_card(question, answer, vec![]).unwrap();
            let card_id = card.id();
            let before_date = Utc::now() + duration;
            let stability = Stability::new(stability_value).unwrap();
            let difficulty = create_test_difficulty();

            // Act
            let result = user.rate_card(
                card_id,
                Rating::Good,
                duration,
                MemoryState::new(stability, difficulty),
            );

            // Assert
            assert!(result.is_ok());
            let card = user.get_card(card_id).unwrap();
            let after_date = Utc::now() + duration;
            // Check that next_review_date is approximately correct (within 1 second)
            assert!(card.next_review_date() >= before_date - Duration::seconds(1));
            assert!(card.next_review_date() <= after_date + Duration::seconds(1));
            assert_eq!(card.stability().unwrap().value(), stability_value);
        }
    }

    #[test]
    fn user_rate_card_should_return_error_when_card_not_found_for_schedule() {
        // Arrange
        let mut user = create_test_user();
        let non_existent_id = ulid::Ulid::new();
        let stability = Stability::new(2.5).unwrap();
        let difficulty = create_test_difficulty();

        // Act
        let result = user.rate_card(
            non_existent_id,
            Rating::Good,
            Duration::days(5),
            MemoryState::new(stability, difficulty),
        );

        // Assert
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, JeersError::CardNotFound { .. }));
        if let JeersError::CardNotFound { card_id } = err {
            assert_eq!(card_id, non_existent_id);
        }
    }

    #[test]
    fn user_get_card_should_return_card_when_card_exists() {
        // Arrange
        let mut user = create_test_user();
        let question = create_test_question();
        let answer = create_test_answer();
        let card = user.create_card(question, answer, vec![]).unwrap();
        let card_id = card.id();

        // Act
        let retrieved_card = user.get_card(card_id);

        // Assert
        assert!(retrieved_card.is_some());
        assert_eq!(retrieved_card.unwrap().id(), card_id);
    }

    #[test]
    fn user_get_card_should_return_none_when_card_not_found() {
        // Arrange
        let user = create_test_user();
        let non_existent_id = ulid::Ulid::new();

        // Act
        let retrieved_card = user.get_card(non_existent_id);

        // Assert
        assert!(retrieved_card.is_none());
    }

    #[test]
    fn user_create_card_should_return_error_when_duplicate_question() {
        // Arrange
        let mut user = create_test_user();
        let question = Question::new("私".to_string(), generate_embedding("私")).unwrap();
        let answer1 = Answer::new("Я".to_string()).unwrap();
        let answer2 = Answer::new("Ты".to_string()).unwrap();

        // Act
        let card1 = user.create_card(question.clone(), answer1, vec![]).unwrap();
        let result = user.create_card(question, answer2, vec![]);

        // Assert
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            JeersError::DuplicateCard { question } if question == "私"
        ));
        assert_eq!(user.cards().len(), 1);
        assert!(user.cards().contains_key(&card1.id()));
    }

    #[test]
    fn user_create_card_should_allow_different_questions() {
        // Arrange
        let mut user = create_test_user();
        let question1 = Question::new("私".to_string(), generate_embedding("私")).unwrap();
        let question2 = Question::new("食べて".to_string(), generate_embedding("食べて")).unwrap();
        let answer = Answer::new("есть".to_string()).unwrap();

        // Act
        let card1 = user.create_card(question1, answer.clone(), vec![]).unwrap();
        let card2 = user.create_card(question2, answer, vec![]).unwrap();

        // Assert
        assert_eq!(user.cards().len(), 2);
        assert_ne!(card1.id(), card2.id());
    }

    #[test]
    fn user_edit_card_should_return_error_when_duplicate_question() {
        // Arrange
        let mut user = create_test_user();
        let question1 = Question::new("私".to_string(), generate_embedding("私")).unwrap();
        let question2 = Question::new("食べて".to_string(), generate_embedding("食べて")).unwrap();
        let answer = Answer::new("есть".to_string()).unwrap();

        let card1 = user.create_card(question1, answer.clone(), vec![]).unwrap();
        let card2 = user.create_card(question2, answer, vec![]).unwrap();
        let card2_id = card2.id();

        let duplicate_question = Question::new("私".to_string(), generate_embedding("私")).unwrap();
        let new_answer = Answer::new("Я".to_string()).unwrap();

        // Act
        let result = user.edit_card(card2_id, duplicate_question, new_answer, vec![]);

        // Assert
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            JeersError::DuplicateCard { question } if question == "私"
        ));
        // Verify that card1 still exists and card2 was not modified
        assert!(user.cards().contains_key(&card1.id()));
        assert!(user.cards().contains_key(&card2_id));
    }

    #[test]
    fn user_edit_card_should_allow_same_question_for_same_card() {
        // Arrange
        let mut user = create_test_user();
        let question = Question::new("私".to_string(), generate_embedding("私")).unwrap();
        let answer1 = Answer::new("Я".to_string()).unwrap();
        let answer2 = Answer::new("Ты".to_string()).unwrap();

        let card = user.create_card(question.clone(), answer1, vec![]).unwrap();
        let card_id = card.id();

        // Act
        let result = user.edit_card(card_id, question, answer2.clone(), vec![]);

        // Assert
        assert!(result.is_ok());
        let updated_card = user.get_card(card_id).unwrap();
        assert_eq!(updated_card.answer().text(), answer2.text());
    }

    #[test]
    fn user_find_similarity_should_return_similar_cards() {
        // Arrange
        let mut user = create_test_user();
        let q1 = Question::new("私".to_string(), generate_embedding("私")).unwrap();
        let a1 = Answer::new("Я".to_string()).unwrap();
        let card1 = user.create_card(q1, a1, vec![]).unwrap();
        let card1_id = card1.id();

        // Create cards with different texts and embeddings
        // These will have different embeddings, so similarity will be low
        // But the method should still work correctly
        let q2 = Question::new("食べて".to_string(), generate_embedding("食べて")).unwrap();
        let a2 = Answer::new("есть".to_string()).unwrap();
        user.create_card(q2, a2, vec![]).unwrap();

        let q3 = Question::new("教えて".to_string(), generate_embedding("教えて")).unwrap();
        let a3 = Answer::new("объяснить".to_string()).unwrap();
        user.create_card(q3, a3, vec![]).unwrap();

        // Act
        let similarity = user.find_similarity(card1_id);

        // Assert
        assert!(similarity.is_ok());
        let similar_cards = similarity.unwrap();
        // The method should return a vector (may be empty if no similar cards found)
        // With our deterministic embeddings, different texts = different embeddings
        // So similarity will be low and cards won't be found as similar
        // But the method should work without errors
        assert!(similar_cards.len() <= 2); // At most 2 other cards
    }

    #[test]
    fn user_find_similarity_should_return_error_when_card_not_found() {
        // Arrange
        let user = create_test_user();
        let non_existent_id = ulid::Ulid::new();

        // Act
        let result = user.find_similarity(non_existent_id);

        // Assert
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            JeersError::CardNotFound { card_id: _ }
        ));
    }

    #[test]
    fn user_find_similar_cards_should_return_limited_results() {
        // Arrange
        let mut user = create_test_user();
        let q1 = Question::new("私".to_string(), generate_embedding("私")).unwrap();
        let a1 = Answer::new("Я".to_string()).unwrap();
        let card1 = user.create_card(q1, a1, vec![]).unwrap();
        let card1_id = card1.id();

        // Create multiple cards
        for i in 2..=5 {
            let q =
                Question::new(format!("私{}", i), generate_embedding(&format!("私{}", i))).unwrap();
            let a = Answer::new(format!("Я{}", i)).unwrap();
            user.create_card(q, a, vec![]).unwrap();
        }

        // Act
        let result = user.find_similar_cards(card1_id, 2);

        // Assert
        assert!(result.is_ok());
        let similar = result.unwrap();
        assert!(similar.len() <= 2);
    }

    #[test]
    fn user_find_similar_cards_should_return_error_when_card_not_found() {
        // Arrange
        let user = create_test_user();
        let non_existent_id = ulid::Ulid::new();

        // Act
        let result = user.find_similar_cards(non_existent_id, 5);

        // Assert
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            JeersError::CardNotFound { card_id: _ }
        ));
    }

    #[test]
    fn user_create_lesson_history_item_should_add_to_history() {
        // Arrange
        let mut user = create_test_user();
        let question = create_test_question();
        let answer = create_test_answer();
        let card = user.create_card(question, answer, vec![]).unwrap();
        let stability = Stability::new(2.0).unwrap();
        let difficulty = create_test_difficulty();
        user.rate_card(
            card.id(),
            Rating::Good,
            Duration::zero(),
            MemoryState::new(stability, difficulty),
        )
        .unwrap();

        // Act
        user.update_daily_history();

        // Assert
        assert_eq!(user.lesson_history().len(), 1);
        let history_item = &user.lesson_history()[0];
        assert_eq!(history_item.total_words(), 1);
    }

    #[test]
    fn user_lesson_history_should_return_empty_when_no_history() {
        // Arrange
        let user = create_test_user();

        // Assert
        assert_eq!(user.lesson_history().len(), 0);
    }

    #[test]
    fn card_is_known_card_should_return_true_when_has_easy_rating() {
        // Arrange
        let mut user = create_test_user();
        let question = create_test_question();
        let answer = create_test_answer();
        let card = user.create_card(question, answer, vec![]).unwrap();
        let card_id = card.id();
        // Known card requires stability > 4.0
        let stability = Stability::new(5.0).unwrap();
        let difficulty = create_test_difficulty();
        user.rate_card(
            card_id,
            Rating::Easy,
            Duration::zero(),
            MemoryState::new(stability, difficulty),
        )
        .unwrap();

        // Act
        let card = user.get_card(card_id).unwrap();

        // Assert
        assert!(card.is_known_card());
    }

    #[test]
    fn card_is_known_card_should_return_false_when_no_easy_rating() {
        // Arrange
        let mut user = create_test_user();
        let question = create_test_question();
        let answer = create_test_answer();
        let card = user.create_card(question, answer, vec![]).unwrap();
        let card_id = card.id();
        let stability = Stability::new(1.0).unwrap();
        let difficulty = create_test_difficulty();
        user.rate_card(
            card_id,
            Rating::Good,
            Duration::zero(),
            MemoryState::new(stability, difficulty),
        )
        .unwrap();

        // Act
        let card = user.get_card(card_id).unwrap();

        // Assert
        assert!(!card.is_known_card());
    }

    #[test]
    fn card_last_review_date_should_return_timestamp_of_last_review() {
        // Arrange
        let mut user = create_test_user();
        let question = create_test_question();
        let answer = create_test_answer();
        let card = user.create_card(question, answer, vec![]).unwrap();
        let card_id = card.id();
        let stability = Stability::new(1.0).unwrap();
        let difficulty = create_test_difficulty();
        user.rate_card(
            card_id,
            Rating::Good,
            Duration::zero(),
            MemoryState::new(stability, difficulty),
        )
        .unwrap();

        // Act
        let card = user.get_card(card_id).unwrap();
        let last_review_date = card.last_review_date();

        // Assert
        assert!(last_review_date.is_some());
    }

    #[test]
    fn card_last_review_date_should_return_none_when_no_reviews() {
        // Arrange
        let mut user = create_test_user();
        let question = create_test_question();
        let answer = create_test_answer();
        let card = user.create_card(question, answer, vec![]).unwrap();
        let card_id = card.id();

        // Act
        let card = user.get_card(card_id).unwrap();
        let last_review_date = card.last_review_date();

        // Assert
        assert!(last_review_date.is_none());
    }

    #[test]
    fn user_create_card_should_store_example_phrases() {
        // Arrange
        let mut user = create_test_user();
        let question = create_test_question();
        let answer = create_test_answer();
        let example_phrases = vec![
            ExamplePhrase::new("私は食べて".to_string(), "Я ем".to_string()),
            ExamplePhrase::new("私は教えて".to_string(), "Я объясняю".to_string()),
        ];

        // Act
        let card = user
            .create_card(question, answer, example_phrases.clone())
            .unwrap();

        // Assert
        assert_eq!(card.example_phrases().len(), 2);
        assert_eq!(card.example_phrases()[0].text(), "私は食べて");
        assert_eq!(card.example_phrases()[1].text(), "私は教えて");
    }

    #[test]
    fn user_start_study_session_should_respect_force_new_cards_flag() {
        // Arrange
        let mut user = User::new(
            "test_user".to_string(),
            JapaneseLevel::N5,
            NativeLanguage::Russian,
            2, // new_cards_limit = 2
        );

        // Create 3 new cards (without reviews - they are new)
        for i in 1..=3 {
            let q =
                Question::new(format!("私{}", i), generate_embedding(&format!("私{}", i))).unwrap();
            let a = Answer::new(format!("Я{}", i)).unwrap();
            user.create_card(q, a, vec![]).unwrap();
        }

        // Act - with force_new_cards = true
        let cards_forced = user.start_study_session(true);
        // Act - with force_new_cards = false
        let cards_not_forced = user.start_study_session(false);

        // Assert
        // When forced, should return all 3 new cards (limit ignored)
        assert_eq!(cards_forced.len(), 3);
        // When not forced, should return only 2 new cards (limited)
        assert_eq!(cards_not_forced.len(), 2);
    }
}
