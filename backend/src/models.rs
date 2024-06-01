use crate::schema::{archer_additions, archers};
use diesel::prelude::*;
use serde::Serialize;

#[derive(Queryable, Selectable, Identifiable, Serialize, Debug, PartialEq, Clone)]
#[diesel(primary_key(bib))]
pub struct Archer {
    pub bib: i64,
    pub session: i32,
    pub division: String,
    pub class: String,
    pub target: String,
    pub individual_qualification: i32,
    pub team_qualification: i32,
    pub individual_final: i32,
    pub team_final: i32,
    pub mixed_team_final: i32,
    pub last_name: String,
    pub first_name: String,
    pub gender: Option<i32>,
    pub country_code: String,
    pub country_name: String,
    pub date_of_birth: String,
    pub subclass: Option<String>,
    pub country_code_2: Option<String>,
    pub country_name_2: Option<String>,
    pub country_code_3: Option<String>,
    pub country_name_3: Option<String>,
}

#[derive(Insertable, Default, Debug, PartialEq)]
#[diesel(table_name = archers)]
pub struct InsertableArcher {
    pub session: i32,
    pub division: String,
    pub class: String,
    pub target: String,
    pub individual_qualification: i32,
    pub team_qualification: i32,
    pub individual_final: i32,
    pub team_final: i32,
    pub mixed_team_final: i32,
    pub last_name: String,
    pub first_name: String,
    pub gender: Option<i32>,
    pub country_code: String,
    pub country_name: String,
    pub date_of_birth: String,
    pub subclass: Option<String>,
    pub country_code_2: String,
    pub country_name_2: String,
    pub country_code_3: String,
    pub country_name_3: String,
}

#[derive(
    Insertable, Queryable, Selectable, Associations, Identifiable, Serialize, Debug, PartialEq,
)]
#[diesel(primary_key(bib))]
#[diesel(belongs_to(Archer, foreign_key = bib))]
#[diesel(table_name = archer_additions)]
pub struct ArcherAdditions {
    pub bib: i64,
    pub email: Option<String>,
    pub comment: Option<String>,
    pub target_face: Option<String>,
}
