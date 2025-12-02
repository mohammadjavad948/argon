// use std::ops::Deref;

use std::fmt::Display;

use sea_orm::FromJsonQueryResult;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult)]
pub struct MultilangField(pub Vec<LangField>);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LangField {
    pub lang: String,
    pub content: String
}

impl MultilangField {
    pub fn new(fileds: Vec<LangField>) -> Self {
        MultilangField(fileds)
    }

    pub fn get_language(&self, language: &str) -> Option<&LangField> {
        self.0.iter().find(|x| x.lang == language)
    }
}

impl LangField {
    pub fn new(lang: String, content: String) -> Self {
        LangField { lang, content }
    }
}

impl Display for LangField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.content)
    }
}

// idk? should i?
// impl Deref for LangField {
//     type Target = str;
// 
//     fn deref(&self) -> &Self::Target {
//         &self.content
//     }
// }
