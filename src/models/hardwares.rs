use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{database::run_query_dsl_ext::RunQueryDslExt, schema::hardwares};

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
        hardwares::table.load_all()
    }

    pub fn find_by_id(hardware_id: i32) -> QueryResult<Hardware> {
        hardwares::table.find(hardware_id).get_first()
    }

    pub fn insert(hardware: HardwareDTO) -> QueryResult<Hardware> {
        diesel::insert_into(hardwares::table)
            .values(hardware)
            .get_result_query()
    }

    pub fn update(hardware_id: i32, hardware: HardwareDTO) -> QueryResult<Hardware> {
        diesel::update(hardwares::table.find(hardware_id))
            .set(hardware)
            .get_result_query()
    }

    pub fn delete(hardware_id: i32) -> QueryResult<usize> {
        diesel::delete(hardwares::table.find(hardware_id)).execute_query()
    }
}
