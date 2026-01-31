use crate::domain::models::template::{Template, TemplateEnv, TemplateResponse};
use crate::error::{AppError, Result};

pub struct TemplateUsecase {
    templates: Vec<Template>,
}

impl TemplateUsecase {
    pub fn new() -> Self {
        let templates = vec![
            Template {
                id: "wordpress".to_string(),
                name: "WordPress".to_string(),
                description: "The world's most popular website builder.".to_string(),
                icon: "globe".to_string(),
                compose_content: r#"version: '3.8'
services:
  db:
    image: mysql:8.0
    volumes:
      - db_data:/var/lib/mysql
    restart: always
    environment:
      MYSQL_ROOT_PASSWORD: ${MYSQL_ROOT_PASSWORD}
      MYSQL_DATABASE: wordpress
      MYSQL_USER: wordpress
      MYSQL_PASSWORD: ${MYSQL_PASSWORD}

  wordpress:
    depends_on:
      - db
    image: wordpress:latest
    ports:
      - "8080:80"
    restart: always
    environment:
      WORDPRESS_DB_HOST: db
      WORDPRESS_DB_USER: wordpress
      WORDPRESS_DB_PASSWORD: ${MYSQL_PASSWORD}
      WORDPRESS_DB_NAME: wordpress

volumes:
  db_data: {}"#.to_string(),
                default_env: vec![
                    TemplateEnv {
                        key: "MYSQL_ROOT_PASSWORD".to_string(),
                        value: "somewordpress".to_string(),
                        description: Some("Root password for MySQL".to_string()),
                    },
                    TemplateEnv {
                        key: "MYSQL_PASSWORD".to_string(),
                        value: "wordpress".to_string(),
                        description: Some("Database password for WordPress".to_string()),
                    },
                ],
            },
            Template {
                id: "ghost".to_string(),
                name: "Ghost".to_string(),
                description: "A professional open source publishing platform.".to_string(),
                icon: "file-text".to_string(),
                compose_content: r#"version: '3.8'
services:
  ghost:
    image: ghost:latest
    restart: always
    ports:
      - "2368:2368"
    environment:
      url: http://localhost:2368
      database__client: sqlite3"#.to_string(),
                default_env: vec![],
            },
            Template {
                id: "redis".to_string(),
                name: "Redis".to_string(),
                description: "In-memory data structure store, used as a database, cache, and message broker.".to_string(),
                icon: "database".to_string(),
                compose_content: r#"version: '3.8'
services:
  redis:
    image: redis:alpine
    ports:
      - "6379:6379"
    restart: always"#.to_string(),
                default_env: vec![],
            },
        ];

        Self { templates }
    }

    pub async fn list_templates(&self) -> Result<Vec<TemplateResponse>> {
        Ok(self.templates.iter().cloned().map(TemplateResponse::from).collect())
    }

    pub async fn get_template(&self, id: &str) -> Result<Template> {
        self.templates
            .iter()
            .find(|t| t.id == id)
            .cloned()
            .ok_or_else(|| AppError::NotFound(format!("Template {} not found", id)))
    }
}
