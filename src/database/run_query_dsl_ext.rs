use crate::database;
use diesel::associations::HasTable;
use diesel::dsl::Limit;
use diesel::pg::PgConnection;
use diesel::query_builder::InsertStatement;
use diesel::query_dsl::methods::ExecuteDsl;
use diesel::query_dsl::methods::LimitDsl;
use diesel::query_dsl::methods::LoadQuery;
use diesel::Insertable;
use diesel::QueryResult;
use diesel::RunQueryDsl;
use diesel::Table;

pub trait RunQueryDslExt: RunQueryDsl<PgConnection> + Sized {
    fn load_all<U>(self) -> QueryResult<Vec<U>>
    where
        for<'a> Self: LoadQuery<'a, PgConnection, U>,
    {
        let mut conn = database::get_conn();
        self.load(&mut conn)
    }

    fn get_first<'a, U>(self) -> QueryResult<U>
    where
        Self: LimitDsl,
        Limit<Self>: LoadQuery<'a, PgConnection, U>,
    {
        let mut conn = database::get_conn();
        self.first(&mut conn)
    }

    fn get_result_query<'a, U>(self) -> QueryResult<U>
    where
        Self: LoadQuery<'a, PgConnection, U>,
    {
        let mut conn = database::get_conn();
        RunQueryDsl::get_result(self, &mut conn)
    }

    fn execute_query(self) -> QueryResult<usize>
    where
        Self: ExecuteDsl<PgConnection>,
    {
        let mut conn = database::get_conn();
        self.execute(&mut conn)
    }

    fn insert<'a, V, T, R>(self, new_record: &'a V) -> QueryResult<R>
    where
        Self: HasTable<Table = T> + Copy,
        V: Insertable<T> + 'a + Clone,
        T: Table,
        InsertStatement<T, V::Values>: LoadQuery<'a, PgConnection, R>,
    {
        let mut conn = database::get_conn();
        diesel::insert_into(Self::table())
            .values(new_record.clone())
            .get_result(&mut conn)
    }
}

impl<T> RunQueryDslExt for T where T: RunQueryDsl<PgConnection> {}
