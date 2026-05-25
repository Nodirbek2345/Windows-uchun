use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DetectionType {
    Phone,
    Email,
    CardNumber,
    Name, // Simple regex based name, for pro use NLP could be plugged in
    Passport,
    Pinfl,
    Inn,
    Date,
    Blacklist,
}

impl DetectionType {
    pub fn as_label(&self) -> &'static str {
        match self {
            DetectionType::Phone => "[TELEFON]",
            DetectionType::Email => "[EMAIL]",
            DetectionType::CardNumber => "[KARTA]",
            DetectionType::Name => "[ISM]",
            DetectionType::Passport => "[PASSPORT]",
            DetectionType::Pinfl => "[PINFL]",
            DetectionType::Inn => "[INN]",
            DetectionType::Date => "[SANA]",
            DetectionType::Blacklist => "[MAXFIY]",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Detection {
    pub dtype: DetectionType,
    pub original_value: String,
    pub start: usize,
    pub end: usize,
    pub confidence: f32,
    pub rule: String,
}

// Regex patterns based on Uzbek standards
static REGEX_PHONE_UZ: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\+998\s?[0-9]{2}\s?[0-9]{3}\s?[0-9]{2}\s?[0-9]{2})").unwrap());
static REGEX_PHONE_INTL: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\+[1-9][0-9]{7,14})").unwrap());
static REGEX_EMAIL: Lazy<Regex> = Lazy::new(|| Regex::new(r"([a-zA-Z0-9._%+\-]+@[a-zA-Z0-9.\-]+\.[a-zA-Z]{2,})").unwrap());
static REGEX_CARD: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b([0-9]{4}[\s\-]?[0-9]{4}[\s\-]?[0-9]{4}[\s\-]?[0-9]{4})\b").unwrap());
static REGEX_PINFL: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b([0-9]{14})\b").unwrap());
static REGEX_PASSPORT: Lazy<Regex> = Lazy::new(|| Regex::new(r"([A-Za-z]{2}[0-9]{7})").unwrap());
static REGEX_INN: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b([0-9]{9})\b").unwrap());
static REGEX_DATE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b(\d{1,2}[./-]\d{1,2}[./-]\d{2,4})\b").unwrap());

pub struct Detector;

impl Detector {
    pub fn new() -> Self {
        Self {}
    }

    pub fn detect_text(&self, text: &str, blacklist: &[String]) -> Vec<Detection> {
        let mut detections = Vec::new();

        self.apply_regex(&REGEX_PHONE_UZ, text, DetectionType::Phone, "uz_phone", &mut detections);
        self.apply_regex(&REGEX_PHONE_INTL, text, DetectionType::Phone, "intl_phone", &mut detections);
        self.apply_regex(&REGEX_EMAIL, text, DetectionType::Email, "email", &mut detections);
        self.apply_regex(&REGEX_CARD, text, DetectionType::CardNumber, "card", &mut detections);
        self.apply_regex(&REGEX_PINFL, text, DetectionType::Pinfl, "pinfl", &mut detections);
        self.apply_regex(&REGEX_PASSPORT, text, DetectionType::Passport, "passport", &mut detections);
        self.apply_regex(&REGEX_INN, text, DetectionType::Inn, "inn", &mut detections);
        self.apply_regex(&REGEX_DATE, text, DetectionType::Date, "date", &mut detections);

        // Very naive Name regex - typically needs NLP or exact dictionary
        // Just matching 2 capitalized words for basic testing
        let regex_name = Regex::new(r"\b([A-Z][a-z]+ [A-Z][a-z]+)\b").unwrap();
        self.apply_regex(&regex_name, text, DetectionType::Name, "capitalized_words", &mut detections);

        // Blacklist matching
        for word in blacklist {
            let pattern = format!(r"(?i)\b({})\b", regex::escape(word));
            if let Ok(re) = Regex::new(&pattern) {
                self.apply_regex(&re, text, DetectionType::Blacklist, "blacklist", &mut detections);
            }
        }

        // Sort by start position descending to allow backward replacement without index shifting
        detections.sort_by(|a, b| b.start.cmp(&a.start));
        detections
    }

    fn apply_regex(&self, re: &Regex, text: &str, dtype: DetectionType, rule_name: &str, result: &mut Vec<Detection>) {
        for cap in re.captures_iter(text) {
            if let Some(m) = cap.get(1) {
                result.push(Detection {
                    dtype: dtype.clone(),
                    original_value: m.as_str().to_string(),
                    start: m.start(),
                    end: m.end(),
                    confidence: 0.9, // Default generic confidence
                    rule: rule_name.to_string(),
                });
            }
        }
    }
}
