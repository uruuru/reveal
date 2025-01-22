use chrono::{Datelike, NaiveDateTime};
use rand::Rng;

#[derive(Default)]
pub struct QuestionAndAnswers {
    pub question: String,
    pub answers: Vec<String>,
    pub idx_correct: usize,
}

pub fn simple_year_question(date: &NaiveDateTime) -> QuestionAndAnswers {
    let year_taken = date.date().year();
    let year_now = chrono::Utc::now().year();
    let number_of_answers = 3;
    let max_offset = std::cmp::min(number_of_answers - 1, year_now - year_taken);
    let offset = rand::thread_rng().gen_range(0..=max_offset);
    QuestionAndAnswers {
        question: "Which year was the picture taken?".into(),
        answers: (year_taken - (number_of_answers - 1) + offset..=year_taken + offset)
            .map(|i| i.to_string())
            .collect(),
        idx_correct: ((number_of_answers - 1) - offset) as usize,
    }
}
