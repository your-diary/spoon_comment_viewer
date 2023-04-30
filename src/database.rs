use rusqlite::{params, Connection, Statement};

#[derive(Debug, PartialEq, Clone, Copy)]
struct ListenerEntity {
    id: usize,
    visit_count: usize,
    stay_duration_sec: usize,
}

impl ListenerEntity {
    fn new(id: usize, visit_count: usize, stay_duration_sec: usize) -> ListenerEntity {
        ListenerEntity {
            id,
            visit_count,
            stay_duration_sec,
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
                    "CREATE TABLE {} (
              id                INTEGER PRIMARY KEY,
              visit_count       INTEGER,
              stay_duration_sec INTEGER
        )",
                    self.table_name
                ),
                [],
            )
            .unwrap();
    }

    fn insert(&self, entity: ListenerEntity) {
        self.conn
            .execute(
                &format!(
                    "INSERT INTO {} (id, visit_count, stay_duration_sec) VALUES (?, ?, ?);",
                    self.table_name
                ),
                params![entity.id, entity.visit_count, entity.stay_duration_sec,],
            )
            .unwrap();
    }

    fn update(&self, entity: ListenerEntity) {
        self.conn
            .execute(
                &format!(
                    "UPDATE {} SET visit_count = ?, stay_duration_sec = ? WHERE id = ?;",
                    self.table_name
                ),
                params![entity.visit_count, entity.stay_duration_sec, entity.id,],
            )
            .unwrap();
    }

    fn select_by_id(&self, id: usize) -> Option<ListenerEntity> {
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
                ))
            })
            .unwrap()
            .map(|i| i.unwrap())
            .collect();
        listeners.get(0).copied()
    }

    fn select_all(&self) -> Vec<ListenerEntity> {
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

        let mut entity1 = ListenerEntity::new(1, 2, 3);
        let entity2 = ListenerEntity::new(10, 20, 30);
        db.insert(entity1);
        db.insert(entity2);

        assert_eq!(vec![entity1, entity2], db.select_all());
        assert_eq!(None, db.select_by_id(0));
        assert_eq!(Some(entity1), db.select_by_id(1));
        assert_eq!(Some(entity2), db.select_by_id(10));
        println!("{:?}", db.select_all());

        entity1.visit_count *= 100;
        entity1.stay_duration_sec *= 100;
        db.update(entity1);
        assert_eq!(vec![entity1, entity2], db.select_all());
        assert_eq!(Some(entity1), db.select_by_id(1));
        assert_eq!(Some(entity2), db.select_by_id(10));
        println!("{:?}", db.select_all());
    }
}
