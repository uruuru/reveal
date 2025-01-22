use chrono::{Datelike, NaiveDateTime};
use rand::Rng;

#[derive(Default)]
pub struct QuestionAndAnswers {
    pub question: String,
    pub answers: Vec<String>,
    pub idx_correct: usize,
}

pub fn simple_year_question(date: &NaiveDateTime) -> QuestionAndAnswers {
    let year = date.date().year();
    let offset = rand::thread_rng().gen_range(0..2);
    QuestionAndAnswers {
        question: "Which year was the picture taken?".into(),
        answers: (year - offset..year - offset + 3)
            .map(|i| i.to_string())
            .collect(),
        idx_correct: offset as usize,
    }
}
