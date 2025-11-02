pub trait Card {
    fn deck_name(&self) -> &str;
    fn tags(&self) -> &[String];
    fn anki_id(&self) -> Option<i64>;
    fn set_anki_id(&mut self, id: i64);

    /// Get raw markdown fields (for ID injection)
    fn raw_fields(&self) -> Vec<&str>;

    /// Get HTML fields ready for Anki
    fn html_fields(&self) -> Vec<String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_card_trait_when_implemented_then_provides_common_interface() {
        // This test just verifies compilation
        fn accepts_card<C: Card>(_card: &C) {
            // Any type implementing Card should work
        }

        let basic = BasicCard::new("Q", "A");
        accepts_card(&basic);
    }

    #[test]
    fn given_front_and_back_when_creating_basic_card_then_stores_fields() {
        let card = BasicCard::new("Question?", "Answer!");

        assert_eq!(card.front_md(), "Question?");
        assert_eq!(card.back_md(), "Answer!");
        assert_eq!(card.deck_name(), "Default");
    }

    #[test]
    fn given_basic_card_when_setting_deck_then_updates() {
        let card = BasicCard::new("Q", "A").with_deck("MyDeck");

        assert_eq!(card.deck_name(), "MyDeck");
    }

    #[test]
    fn given_basic_card_when_setting_tags_then_stores() {
        let card = BasicCard::new("Q", "A").with_tags(vec!["tag1".to_string(), "tag2".to_string()]);

        assert_eq!(card.tags(), &["tag1", "tag2"]);
    }

    #[test]
    fn given_text_with_cloze_when_creating_then_stores_text() {
        let card = ClozeCard::new("The capital of {{c1::France}} is {{c2::Paris}}");

        assert_eq!(
            card.text_md(),
            "The capital of {{c1::France}} is {{c2::Paris}}"
        );
    }

    #[test]
    fn given_cloze_card_when_implementing_trait_then_provides_interface() {
        let card = ClozeCard::new("Text {{c1::cloze}}").with_deck("TestDeck");

        assert_eq!(card.deck_name(), "TestDeck");
        assert_eq!(card.raw_fields(), vec!["Text {{c1::cloze}}"]);
    }
}

#[derive(Debug, Clone)]
pub struct BasicCard {
    front_md: String,
    back_md: String,
    front_html: Option<String>,
    back_html: Option<String>,
    tags: Vec<String>,
    deck_name: String,
    anki_id: Option<i64>,
}

impl BasicCard {
    pub fn new(front: impl Into<String>, back: impl Into<String>) -> Self {
        Self {
            front_md: front.into(),
            back_md: back.into(),
            front_html: None,
            back_html: None,
            tags: Vec::new(),
            deck_name: "Default".to_string(),
            anki_id: None,
        }
    }

    pub fn with_deck(mut self, deck: impl Into<String>) -> Self {
        self.deck_name = deck.into();
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn with_id(mut self, id: i64) -> Self {
        self.anki_id = Some(id);
        self
    }

    pub fn front_md(&self) -> &str {
        &self.front_md
    }

    pub fn back_md(&self) -> &str {
        &self.back_md
    }

    pub fn set_html(&mut self, front: String, back: String) {
        self.front_html = Some(front);
        self.back_html = Some(back);
    }
}

impl Card for BasicCard {
    fn deck_name(&self) -> &str {
        &self.deck_name
    }

    fn tags(&self) -> &[String] {
        &self.tags
    }

    fn anki_id(&self) -> Option<i64> {
        self.anki_id
    }

    fn set_anki_id(&mut self, id: i64) {
        self.anki_id = Some(id);
    }

    fn raw_fields(&self) -> Vec<&str> {
        vec![&self.front_md, &self.back_md]
    }

    fn html_fields(&self) -> Vec<String> {
        vec![
            self.front_html
                .clone()
                .unwrap_or_else(|| self.front_md.clone()),
            self.back_html
                .clone()
                .unwrap_or_else(|| self.back_md.clone()),
        ]
    }
}

#[derive(Debug, Clone)]
pub struct ClozeCard {
    text_md: String,
    text_html: Option<String>,
    tags: Vec<String>,
    deck_name: String,
    anki_id: Option<i64>,
}

impl ClozeCard {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text_md: text.into(),
            text_html: None,
            tags: Vec::new(),
            deck_name: "Default".to_string(),
            anki_id: None,
        }
    }

    pub fn with_deck(mut self, deck: impl Into<String>) -> Self {
        self.deck_name = deck.into();
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn with_id(mut self, id: i64) -> Self {
        self.anki_id = Some(id);
        self
    }

    pub fn text_md(&self) -> &str {
        &self.text_md
    }

    pub fn set_html(&mut self, html: String) {
        self.text_html = Some(html);
    }

    /// Update the markdown text (used during cloze syntax conversion)
    pub fn update_text_md(&mut self, text: String) {
        self.text_md = text;
    }
}

impl Card for ClozeCard {
    fn deck_name(&self) -> &str {
        &self.deck_name
    }

    fn tags(&self) -> &[String] {
        &self.tags
    }

    fn anki_id(&self) -> Option<i64> {
        self.anki_id
    }

    fn set_anki_id(&mut self, id: i64) {
        self.anki_id = Some(id);
    }

    fn raw_fields(&self) -> Vec<&str> {
        vec![&self.text_md]
    }

    fn html_fields(&self) -> Vec<String> {
        vec![self
            .text_html
            .clone()
            .unwrap_or_else(|| self.text_md.clone())]
    }
}
