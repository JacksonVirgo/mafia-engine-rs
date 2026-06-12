use anyhow::Context;
use sqlx::{
    FromRow, MySql, QueryBuilder,
    types::chrono::{DateTime, Utc},
};

use crate::Database;

#[derive(Debug, Clone, FromRow)]
pub struct Server {
    pub server_id: u64,
    pub created_at: DateTime<Utc>,
}

pub enum SelectServer {
    Id(u64),
    WithFlags((u64, Vec<String>)),
}

impl Server {
    pub async fn select(db: &Database, query: SelectServer) -> anyhow::Result<Server> {
        match query {
            SelectServer::Id(id) => {
                sqlx::query_as!(Server, "SELECT * FROM servers WHERE server_id = ?", id)
                    .fetch_one(db)
                    .await
                    .with_context(|| format!("failed to select server with id {id}"))
            }

            SelectServer::WithFlags((id, flags)) => {
                if flags.is_empty() {
                    return Box::pin(Self::select(db, SelectServer::Id(id))).await;
                }
                let mut qb = QueryBuilder::<MySql>::new(
                    "SELECT s.server_id, s.created_at FROM servers s \
                     WHERE s.server_id = ",
                );
                qb.push_bind(id);
                qb.push(
                    " AND (SELECT COUNT(*) FROM server_flags \
                     WHERE server_id = s.server_id AND flag IN (",
                );
                let mut sep = qb.separated(", ");
                for flag in &flags {
                    sep.push_bind(flag);
                }
                qb.push(")) = ");
                qb.push_bind(flags.len() as u64);
                qb.build_query_as::<Server>()
                    .fetch_one(db)
                    .await
                    .with_context(|| {
                        format!("failed to select server with id {id} and flags {flags:?}")
                    })
            }
        }
    }
}
