// @generated automatically by Diesel CLI.

diesel::table! {
    archer_additions (bib) {
        bib -> BigInt,
        email -> Nullable<Text>,
        comment -> Nullable<Text>,
        #[sql_name = "target face"]
        target_face -> Nullable<Text>,
    }
}

diesel::table! {
    archers (bib) {
        bib -> BigInt,
        session -> Integer,
        division -> Text,
        class -> Text,
        target -> Text,
        #[sql_name = "individual qualification"]
        individual_qualification -> Integer,
        #[sql_name = "team qualification"]
        team_qualification -> Integer,
        #[sql_name = "individual final"]
        individual_final -> Integer,
        #[sql_name = "team final"]
        team_final -> Integer,
        #[sql_name = "mixed team final"]
        mixed_team_final -> Integer,
        #[sql_name = "last name"]
        last_name -> Text,
        #[sql_name = "first name"]
        first_name -> Text,
        gender -> Nullable<Integer>,
        #[sql_name = "country code"]
        country_code -> Text,
        #[sql_name = "country name"]
        country_name -> Text,
        #[sql_name = "date of birth"]
        date_of_birth -> Text,
        subclass -> Nullable<Text>,
        #[sql_name = "country code 2"]
        country_code_2 -> Nullable<Text>,
        #[sql_name = "country name 2"]
        country_name_2 -> Nullable<Text>,
        #[sql_name = "country code 3"]
        country_code_3 -> Nullable<Text>,
        #[sql_name = "country name 3"]
        country_name_3 -> Nullable<Text>,
    }
}

diesel::joinable!(archer_additions -> archers (bib));

diesel::allow_tables_to_appear_in_same_query!(archer_additions, archers,);
