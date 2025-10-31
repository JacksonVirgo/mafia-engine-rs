use crate::prelude::Database;

pub struct SignupBuilder<'a> {
    pub message_id: u64,
    pub name: String,
    pub categories: Vec<&'a mut CategoryBuilder>,
}

impl<'a> SignupBuilder<'a> {
    pub fn new(message_id: u64) -> Self {
        Self {
            message_id,
            name: "Signup".into(),
            categories: vec![],
        }
    }

    pub fn name(&mut self, name: impl Into<String>) -> &mut Self {
        self.name = name.into();
        self
    }

    pub fn add_category(&mut self, category: &'a mut CategoryBuilder) -> &mut Self {
        self.categories.push(category);
        self
    }

    pub fn add_categories(&mut self, categories: Vec<&'a mut CategoryBuilder>) -> &mut Self {
        self.categories.extend(categories);
        self
    }

    pub async fn insert_in_db(&self, db: &Database) -> anyhow::Result<()> {
        let mut tx = db.begin().await?;

        sqlx::query!(
            "INSERT INTO signups (message_id, name) VALUES (?, ?)",
            self.message_id,
            self.name
        )
        .execute(&mut *tx)
        .await?;

        for cat in self.categories.iter() {
            sqlx::query!(
                r#"INSERT INTO signup_categories (
                signup_id,
                name,
                button_name,
                max_slots,
                is_hoisted,
                is_anonymous,
                allows_hydra_slots,
                allows_single_slots
            ) VALUES (
                ?, ?, ?, ?, ?, ?, ?, ?
            )"#,
                self.message_id,
                cat.name,
                cat.button_name,
                cat.max_slots,
                cat.is_hoisted,
                cat.is_anonymous,
                cat.allows_hydra_slots,
                cat.allows_single_slots
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(())
    }
}

pub struct CategoryBuilder {
    pub name: String,
    pub button_name: String,
    pub max_slots: Option<u8>,
    pub is_hoisted: bool,
    pub is_anonymous: bool,
    pub allows_hydra_slots: bool,
    pub allows_single_slots: bool,
}

impl Default for CategoryBuilder {
    fn default() -> Self {
        Self {
            name: "Category".into(),
            button_name: "Join Category".into(),
            max_slots: None,
            is_hoisted: false,
            is_anonymous: false,
            allows_hydra_slots: false,
            allows_single_slots: true,
        }
    }
}

impl CategoryBuilder {
    pub fn name(&mut self, name: impl Into<String>) -> &mut Self {
        self.name = name.into();
        self
    }

    pub fn button_name(&mut self, name: impl Into<String>) -> &mut Self {
        self.button_name = name.into();
        self
    }

    pub fn max_slots(&mut self, max_slots: Option<u8>) -> &mut Self {
        self.max_slots = max_slots;
        self
    }

    pub fn set_hoisted(&mut self, hoisted: bool) -> &mut Self {
        self.is_hoisted = hoisted;
        self
    }

    pub fn set_anonymous(&mut self, anonymous: bool) -> &mut Self {
        self.is_anonymous = anonymous;
        self
    }

    pub fn set_allow_hydras(&mut self, allow: bool) -> &mut Self {
        self.allows_hydra_slots = allow;
        self
    }

    pub fn set_allow_singles(&mut self, allow: bool) -> &mut Self {
        self.allows_single_slots = allow;
        self
    }
}
