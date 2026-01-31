use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub compose_content: String,
    pub default_env: Vec<TemplateEnv>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateEnv {
    pub key: String,
    pub value: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateResponse {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
}

impl From<Template> for TemplateResponse {
    fn from(t: Template) -> Self {
        Self {
            id: t.id,
            name: t.name,
            description: t.description,
            icon: t.icon,
        }
    }
}
