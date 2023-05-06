use std::time::Duration;

use rusqlite::{params, Connection, Statement};

#[derive(Debug, PartialEq, Clone)]
pub struct ListenerEntity {
    pub id: usize,
    pub name: String,
    pub visit_count: usize,
    pub stay_duration: Duration,
}

impl ListenerEntity {
    pub fn new(
        id: usize,
        name: String,
        visit_count: usize,
        stay_duration: Duration,
    ) -> ListenerEntity {
        ListenerEntity {
            id,
            name,
            visit_count,
            stay_duration,
        }
    }
}

const TABLE_NAME: &str = "listeners";

pub struct Database {
    conn: Connection,
    table_name: &'static str,
}

impl Database {
    pub fn new(db_path: Option<&str>) -> Self {
        let conn = if let Some(s) = db_path {
            Connection::open(s).unwrap()
        } else {
            Connection::open_in_memory().unwrap()
        };

        let ret = Self {
            conn,
            table_name: TABLE_NAME,
        };
        ret.create_table();
        ret
    }

    fn create_table(&self) {
        self.conn
            .execute(
                &format!(
                    "CREATE TABLE IF NOT EXISTS {} (
              id                INTEGER PRIMARY KEY,
              name              TEXT,
              visit_count       INTEGER,
              stay_duration     INTEGER
        )",
                    self.table_name
                ),
                [],
            )
            .unwrap();
    }

    pub fn insert(&self, entity: ListenerEntity) {
        self.conn
            .execute(
                &format!(
                    "INSERT INTO {} (id, name, visit_count, stay_duration) VALUES (?, ?, ?, ?);",
                    self.table_name
                ),
                params![
                    entity.id,
                    entity.name,
                    entity.visit_count,
                    entity.stay_duration.as_secs(),
                ],
            )
            .unwrap();
    }

    pub fn update(&self, entity: ListenerEntity) {
        self.conn
            .execute(
                &format!(
                    "UPDATE {} SET name = ?, visit_count = ?, stay_duration = ? WHERE id = ?;",
                    self.table_name
                ),
                params![
                    entity.name,
                    entity.visit_count,
                    entity.stay_duration.as_secs(),
                    entity.id,
                ],
            )
            .unwrap();
    }

    pub fn select_by_id(&self, id: usize) -> Option<ListenerEntity> {
        let mut statement: Statement = self
            .conn
            .prepare(&format!("SELECT * FROM {} WHERE id = ?;", self.table_name))
            .unwrap();
        let listeners: Vec<ListenerEntity> = statement
            .query_map([id], |r| {
                Ok(ListenerEntity::new(
                    r.get(0).unwrap(),
                    r.get(1).unwrap(),
                    r.get(2).unwrap(),
                    Duration::from_secs(r.get(3).unwrap()),
                ))
            })
            .unwrap()
            .map(|i| i.unwrap())
            .collect();
        listeners.get(0).cloned()
    }

    pub fn select_all(&self) -> Vec<ListenerEntity> {
        let mut statement: Statement = self
            .conn
            .prepare(&format!("SELECT * FROM {};", self.table_name))
            .unwrap();
        statement
            .query_map([], |r| {
                Ok(ListenerEntity::new(
                    r.get(0).unwrap(),
                    r.get(1).unwrap(),
                    r.get(2).unwrap(),
                    Duration::from_secs(r.get(3).unwrap()),
                ))
            })
            .unwrap()
            .map(|i| i.unwrap())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // #[ignore]
    fn test01() {
        let db = Database::new(None);

        let mut entity1 = ListenerEntity::new(1, "Mike".to_string(), 2, Duration::from_secs(3));
        let entity2 = ListenerEntity::new(10, "Lisa".to_string(), 20, Duration::from_secs(30));
        db.insert(entity1.clone());
        db.insert(entity2.clone());

        assert_eq!(vec![entity1.clone(), entity2.clone()], db.select_all());
        assert_eq!(None, db.select_by_id(0));
        assert_eq!(Some(entity1.clone()), db.select_by_id(1));
        assert_eq!(Some(entity2.clone()), db.select_by_id(10));
        println!("{:?}", db.select_all());

        entity1.name = "Bob".to_string();
        entity1.visit_count *= 100;
        entity1.stay_duration *= 100;
        db.update(entity1.clone());
        assert_eq!(vec![entity1.clone(), entity2.clone()], db.select_all());
        assert_eq!(Some(entity1), db.select_by_id(1));
        assert_eq!(Some(entity2), db.select_by_id(10));
        println!("{:?}", db.select_all());
    }
}
