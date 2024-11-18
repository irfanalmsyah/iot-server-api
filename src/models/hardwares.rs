use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{db, schema::hardwares};

#[derive(Queryable, Selectable, Serialize, Deserialize, AsChangeset)]
#[diesel(table_name = crate::schema::hardwares)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Hardware {
    pub id: i32,
    pub name: String,
    pub type_: String,
    pub description: String,
}

#[derive(Insertable, Serialize, Deserialize, AsChangeset)]
#[diesel(table_name = hardwares)]
pub struct HardwareDTO {
    pub name: String,
    pub type_: String,
    pub description: String,
}

impl Hardware {
    pub fn all() -> QueryResult<Vec<Hardware>> {
        let mut conn = db::get_conn();
        hardwares::table.load::<Hardware>(&mut conn)
    }

    pub fn find_by_id(hardware_id: i32) -> QueryResult<Hardware> {
        let mut conn = db::get_conn();
        hardwares::table.find(hardware_id).first(&mut conn)
    }

    pub fn insert(hardware: HardwareDTO) -> QueryResult<Hardware> {
        let mut conn = db::get_conn();
        diesel::insert_into(hardwares::table)
            .values(hardware)
            .get_result(&mut conn)
    }

    pub fn update(hardware_id: i32, hardware: HardwareDTO) -> QueryResult<Hardware> {
        let mut conn = db::get_conn();
        diesel::update(hardwares::table.find(hardware_id))
            .set(hardware)
            .get_result(&mut conn)
    }

    pub fn delete(hardware_id: i32) -> QueryResult<usize> {
        let mut conn = db::get_conn();
        diesel::delete(hardwares::table.find(hardware_id)).execute(&mut conn)
    }
}
