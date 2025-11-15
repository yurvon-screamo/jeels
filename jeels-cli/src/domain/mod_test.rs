#[cfg(test)]
mod tests {
    use crate::domain::User;
    use crate::domain::error::JeersError;
    use crate::domain::value_objects::{
        Answer, Interval, MemoryState, Question, Rating, Stability,
    };
    use chrono::{Duration, Utc};

    fn create_test_user() -> User {
        User::new("test_user".to_string())
    }

    fn create_test_question() -> Question {
        Question::new(
            "What is Rust?".to_string(),
            generate_embedding("What is Rust?"),
        )
        .unwrap()
    }

    fn generate_embedding(text: &str) -> Vec<f32> {
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
        embedding
    }

    fn create_test_answer() -> Answer {
        Answer::new("A systems programming language".to_string()).unwrap()
    }

    fn create_test_memory_state() -> MemoryState {
        MemoryState::new(Stability::new(1.0).unwrap(), 0.5).unwrap()
    }

    #[test]
    fn user_new_should_create_user_with_username_and_empty_cards() {
        // Arrange
        let username = "test_user".to_string();

        // Act
        let user = User::new(username);

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
            let user = User::new(username.to_string());

            // Assert
            assert_eq!(user.username(), username);
        }
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
        let card = user.create_card(question.clone(), answer.clone()).unwrap();

        // Assert
        assert_eq!(card.question().text(), "What is Rust?");
        assert_eq!(card.answer().text(), "A systems programming language");
        assert_eq!(user.cards().len(), 1);
        assert!(user.cards().contains_key(&card.id()));
    }

    #[test]
    fn user_create_card_should_create_multiple_cards_with_unique_ids() {
        // Arrange
        let mut user = create_test_user();
        let test_data = vec![("Q1", "A1"), ("Q2", "A2"), ("Q3", "A3")];

        // Act
        let mut card_ids = Vec::new();
        for (q_text, a_text) in test_data {
            let question = Question::new(q_text.to_string(), generate_embedding(q_text)).unwrap();
            let answer = Answer::new(a_text.to_string()).unwrap();
            let card = user.create_card(question, answer).unwrap();
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
        let card = user.create_card(question, answer).unwrap();
        let card_id = card.id();
        let new_question = Question::new(
            "What is Python?".to_string(),
            generate_embedding("What is Python?"),
        )
        .unwrap();
        let new_answer = Answer::new("A high-level programming language".to_string()).unwrap();

        // Act
        let result = user.edit_card(card_id, new_question.clone(), new_answer.clone());

        // Assert
        assert!(result.is_ok());
        let updated_card = user.get_card(card_id).unwrap();
        assert_eq!(updated_card.question().text(), "What is Python?");
        assert_eq!(
            updated_card.answer().text(),
            "A high-level programming language"
        );
    }

    #[test]
    fn user_edit_card_should_return_error_when_card_not_found() {
        // Arrange
        let mut user = create_test_user();
        let non_existent_id = ulid::Ulid::new();
        let question = create_test_question();
        let answer = create_test_answer();

        // Act
        let result = user.edit_card(non_existent_id, question, answer);

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
        let card = user.create_card(question, answer).unwrap();
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
        let cards = user.start_study_session();

        // Assert
        assert_eq!(cards.len(), 0);
    }

    #[test]
    fn user_start_study_session_should_return_due_cards_when_cards_exist() {
        // Arrange
        let mut user = create_test_user();
        let question = create_test_question();
        let answer = create_test_answer();
        let card = user.create_card(question, answer).unwrap();
        let card_id = card.id();

        // Act
        let cards = user.start_study_session();

        // Assert
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].id(), card_id);
    }

    #[test]
    fn user_start_study_session_should_filter_non_due_cards() {
        // Arrange
        let mut user = create_test_user();
        let question = create_test_question();
        let answer = create_test_answer();
        let card = user.create_card(question, answer).unwrap();
        let card_id = card.id();
        let future_date = Utc::now() + Duration::days(1);
        let stability = Stability::new(1.0).unwrap();
        let memory_state = create_test_memory_state();
        user.schedule_next_review(card_id, future_date, stability, memory_state)
            .unwrap();

        // Act
        let cards = user.start_study_session();

        // Assert
        assert_eq!(cards.len(), 0);
    }

    #[test]
    fn user_start_study_session_should_return_only_due_cards_when_mixed() {
        // Arrange
        let mut user = create_test_user();

        let q1 = Question::new("Q1".to_string(), generate_embedding("Q1")).unwrap();
        let a1 = Answer::new("A1".to_string()).unwrap();
        let card1 = user.create_card(q1, a1).unwrap();
        let card1_id = card1.id();

        let q2 = Question::new("Q2".to_string(), generate_embedding("Q2")).unwrap();
        let a2 = Answer::new("A2".to_string()).unwrap();
        let card2 = user.create_card(q2, a2).unwrap();
        let card2_id = card2.id();

        let future_date = Utc::now() + Duration::days(1);
        let stability = Stability::new(1.0).unwrap();
        let memory_state = create_test_memory_state();
        user.schedule_next_review(card2_id, future_date, stability, memory_state)
            .unwrap();

        // Act
        let cards = user.start_study_session();

        // Assert
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].id(), card1_id);
    }

    #[test]
    fn user_rate_card_should_add_review_when_card_exists() {
        // Arrange
        let mut user = create_test_user();
        let question = create_test_question();
        let answer = create_test_answer();
        let card = user.create_card(question, answer).unwrap();
        let card_id = card.id();
        let rating = Rating::Good;
        let interval = Interval::new(1);

        // Act
        let result = user.rate_card(card_id, rating, interval);

        // Assert
        assert!(result.is_ok());
        let card = user.get_card(card_id).unwrap();
        assert_eq!(card.reviews().len(), 1);
        assert_eq!(card.reviews()[0].rating(), rating);
        assert_eq!(card.reviews()[0].interval().days(), 1);
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
            let card = user.create_card(question, answer).unwrap();
            let card_id = card.id();
            let interval = Interval::new(expected_interval);

            // Act
            let result = user.rate_card(card_id, rating, interval);

            // Assert
            assert!(result.is_ok());
            let card = user.get_card(card_id).unwrap();
            assert_eq!(card.reviews().len(), 1);
            assert_eq!(card.reviews()[0].rating(), rating);
            assert_eq!(card.reviews()[0].interval().days(), expected_interval);
        }
    }

    #[test]
    fn user_rate_card_should_allow_multiple_ratings() {
        // Arrange
        let mut user = create_test_user();
        let question = create_test_question();
        let answer = create_test_answer();
        let card = user.create_card(question, answer).unwrap();
        let card_id = card.id();
        let ratings = vec![(Rating::Easy, 1), (Rating::Good, 2), (Rating::Hard, 3)];

        // Act
        for (rating, interval_days) in ratings.iter() {
            user.rate_card(card_id, *rating, Interval::new(*interval_days))
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
        let interval = Interval::new(1);

        // Act
        let result = user.rate_card(non_existent_id, rating, interval);

        // Assert
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, JeersError::CardNotFound { .. }));
        if let JeersError::CardNotFound { card_id } = err {
            assert_eq!(card_id, non_existent_id);
        }
    }

    #[test]
    fn user_schedule_next_review_should_update_card_when_card_exists() {
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
            let card = user.create_card(question, answer).unwrap();
            let card_id = card.id();
            let future_date = Utc::now() + duration;
            let stability = Stability::new(stability_value).unwrap();
            let memory_state = create_test_memory_state();

            // Act
            let result = user.schedule_next_review(card_id, future_date, stability, memory_state);

            // Assert
            assert!(result.is_ok());
            let card = user.get_card(card_id).unwrap();
            assert_eq!(card.next_review_date(), future_date);
            assert_eq!(card.stability().value(), stability_value);
        }
    }

    #[test]
    fn user_schedule_next_review_should_return_error_when_card_not_found() {
        // Arrange
        let mut user = create_test_user();
        let non_existent_id = ulid::Ulid::new();
        let future_date = Utc::now() + Duration::days(5);
        let stability = Stability::new(2.5).unwrap();
        let memory_state = create_test_memory_state();

        // Act
        let result =
            user.schedule_next_review(non_existent_id, future_date, stability, memory_state);

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
        let card = user.create_card(question, answer).unwrap();
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
        let question = Question::new(
            "What is Rust?".to_string(),
            generate_embedding("What is Rust?"),
        )
        .unwrap();
        let answer1 = Answer::new("A systems programming language".to_string()).unwrap();
        let answer2 = Answer::new("Another answer".to_string()).unwrap();

        // Act
        let card1 = user.create_card(question.clone(), answer1).unwrap();
        let result = user.create_card(question, answer2);

        // Assert
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            JeersError::DuplicateCard { question } if question == "What is Rust?"
        ));
        assert_eq!(user.cards().len(), 1);
        assert!(user.cards().contains_key(&card1.id()));
    }

    #[test]
    fn user_create_card_should_allow_different_questions() {
        // Arrange
        let mut user = create_test_user();
        let question1 = Question::new(
            "What is Rust?".to_string(),
            generate_embedding("What is Rust?"),
        )
        .unwrap();
        let question2 = Question::new(
            "What is Python?".to_string(),
            generate_embedding("What is Python?"),
        )
        .unwrap();
        let answer = Answer::new("A programming language".to_string()).unwrap();

        // Act
        let card1 = user.create_card(question1, answer.clone()).unwrap();
        let card2 = user.create_card(question2, answer).unwrap();

        // Assert
        assert_eq!(user.cards().len(), 2);
        assert_ne!(card1.id(), card2.id());
    }

    #[test]
    fn user_create_card_should_be_case_insensitive_for_duplicates() {
        // Arrange
        let mut user = create_test_user();
        let question1 = Question::new(
            "What is Rust?".to_string(),
            generate_embedding("What is Rust?"),
        )
        .unwrap();
        // Same text (case-insensitive) should have same embedding for duplicate detection
        let question2 = Question::new(
            "what is rust?".to_string(),
            generate_embedding("What is Rust?"),
        )
        .unwrap();
        let answer = Answer::new("A programming language".to_string()).unwrap();

        // Act
        let card1 = user.create_card(question1, answer.clone()).unwrap();
        let result = user.create_card(question2, answer);

        // Assert
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            JeersError::DuplicateCard { question } if question == "what is rust?"
        ));
        assert_eq!(user.cards().len(), 1);
        assert!(user.cards().contains_key(&card1.id()));
    }

    #[test]
    fn user_edit_card_should_return_error_when_duplicate_question() {
        // Arrange
        let mut user = create_test_user();
        let question1 = Question::new(
            "What is Rust?".to_string(),
            generate_embedding("What is Rust?"),
        )
        .unwrap();
        let question2 = Question::new(
            "What is Python?".to_string(),
            generate_embedding("What is Python?"),
        )
        .unwrap();
        let answer = Answer::new("A programming language".to_string()).unwrap();

        let card1 = user.create_card(question1, answer.clone()).unwrap();
        let card2 = user.create_card(question2, answer).unwrap();
        let card2_id = card2.id();

        let duplicate_question = Question::new(
            "What is Rust?".to_string(),
            generate_embedding("What is Rust?"),
        )
        .unwrap();
        let new_answer = Answer::new("New answer".to_string()).unwrap();

        // Act
        let result = user.edit_card(card2_id, duplicate_question, new_answer);

        // Assert
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            JeersError::DuplicateCard { question } if question == "What is Rust?"
        ));
        // Verify that card1 still exists and card2 was not modified
        assert!(user.cards().contains_key(&card1.id()));
        assert!(user.cards().contains_key(&card2_id));
    }

    #[test]
    fn user_edit_card_should_allow_same_question_for_same_card() {
        // Arrange
        let mut user = create_test_user();
        let question = Question::new(
            "What is Rust?".to_string(),
            generate_embedding("What is Rust?"),
        )
        .unwrap();
        let answer1 = Answer::new("A systems programming language".to_string()).unwrap();
        let answer2 = Answer::new("Updated answer".to_string()).unwrap();

        let card = user.create_card(question.clone(), answer1).unwrap();
        let card_id = card.id();

        // Act
        let result = user.edit_card(card_id, question, answer2.clone());

        // Assert
        assert!(result.is_ok());
        let updated_card = user.get_card(card_id).unwrap();
        assert_eq!(updated_card.answer().text(), answer2.text());
    }

    #[test]
    fn user_edit_card_should_trim_and_compare_case_insensitively() {
        // Arrange
        let mut user = create_test_user();
        let question1 = Question::new(
            "What is Rust?".to_string(),
            generate_embedding("What is Rust?"),
        )
        .unwrap();
        let question2 = Question::new(
            "What is Python?".to_string(),
            generate_embedding("What is Python?"),
        )
        .unwrap();
        let answer = Answer::new("A programming language".to_string()).unwrap();

        let card1 = user.create_card(question1, answer.clone()).unwrap();
        let card2 = user.create_card(question2, answer).unwrap();
        let card2_id = card2.id();

        // Same text (trimmed and case-insensitive) should have same embedding
        let duplicate_question = Question::new(
            String::from("  WHAT IS RUST?  "),
            generate_embedding("What is Rust?"),
        )
        .unwrap();
        let new_answer = Answer::new("New answer".to_string()).unwrap();

        // Act
        let result = user.edit_card(card2_id, duplicate_question, new_answer);

        // Assert
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            JeersError::DuplicateCard { question } if question == "  WHAT IS RUST?  "
        ));
        // Verify that card1 still exists and card2 was not modified
        assert!(user.cards().contains_key(&card1.id()));
        assert!(user.cards().contains_key(&card2_id));
    }
}
