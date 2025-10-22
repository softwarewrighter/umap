use anyhow::Result;
use rusqlite::{Connection, OpenFlags, params};
use time::OffsetDateTime;

use crate::types::ChunkRecord;

pub struct Db {
    conn: Connection,
}

impl Db {
    pub fn open(path: &str) -> Result<Self> {
        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
        )?;
        let db = Self { conn };
        db.init()?;
        Ok(db)
    }

    fn init(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            PRAGMA journal_mode=WAL;
            CREATE TABLE IF NOT EXISTS chunks (
                id INTEGER PRIMARY KEY,
                source TEXT NOT NULL,
                chunk_index INTEGER NOT NULL,
                text TEXT NOT NULL,
                dim INTEGER NOT NULL,
                vector BLOB NOT NULL,
                created_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_chunks_source ON chunks(source);
            CREATE INDEX IF NOT EXISTS idx_chunks_chunk_index ON chunks(chunk_index);
            "#,
        )?;
        Ok(())
    }

    pub fn insert_chunk(
        &self,
        source: &str,
        chunk_index: i64,
        text: &str,
        vector: &[f32],
    ) -> Result<i64> {
        let created_at = OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap();
        let dim = vector.len() as i64;
        let mut stmt = self.conn.prepare(
            r#"
            INSERT INTO chunks (source, chunk_index, text, dim, vector, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
        )?;
        let mut blob: Vec<u8> = Vec::with_capacity(vector.len() * 4);
        for v in vector {
            blob.extend_from_slice(&v.to_le_bytes());
        }
        stmt.execute(params![source, chunk_index, text, dim, blob, created_at])?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn all_chunks(&self) -> Result<Vec<ChunkRecord>> {
        let mut stmt = self.conn.prepare(
            r#"SELECT id, source, chunk_index, text, dim, vector FROM chunks ORDER BY id"#,
        )?;
        let rows = stmt.query_map([], |row| {
            let id: i64 = row.get(0)?;
            let source: String = row.get(1)?;
            let chunk_index: i64 = row.get(2)?;
            let text: String = row.get(3)?;
            let dim: i64 = row.get(4)?;
            let blob: Vec<u8> = row.get(5)?;
            let expected = (dim as usize) * 4;
            if blob.len() != expected {
                let err =
                    std::io::Error::new(std::io::ErrorKind::InvalidData, "vector length mismatch");
                return Err(rusqlite::Error::FromSqlConversionFailure(
                    5,
                    rusqlite::types::Type::Blob,
                    Box::new(err),
                ));
            }
            let mut vec = vec![0f32; dim as usize];
            for (i, item) in vec.iter_mut().enumerate().take(dim as usize) {
                let j = i * 4;
                *item = f32::from_le_bytes([blob[j], blob[j + 1], blob[j + 2], blob[j + 3]]);
            }
            Ok(ChunkRecord {
                id,
                source,
                chunk_index,
                text,
                dim: dim as usize,
                vector: vec,
            })
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    pub fn count_chunks(&self) -> Result<i64> {
        let mut stmt = self.conn.prepare("SELECT COUNT(*) FROM chunks")?;
        let cnt: i64 = stmt.query_row([], |row| row.get(0))?;
        Ok(cnt)
    }
}

// helper removed; inline conversion in query_map
