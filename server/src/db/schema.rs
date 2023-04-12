// @generated automatically by Diesel CLI.

diesel::table! {
    completions (id) {
        id -> Int4,
        achieved_mark -> Int4,
        total_marks -> Int4,
        date -> Nullable<Date>,
        comments -> Nullable<Text>,
        test_id -> Int4,
    }
}

diesel::table! {
    tests (id) {
        id -> Int4,
        subject -> Text,
        topic -> Nullable<Text>,
        date_or_id -> Text,
        qualification_level -> Nullable<Text>,
        exam_board -> Nullable<Text>,
        user_id -> Text,
    }
}

diesel::table! {
    users (id) {
        id -> Text,
        username -> Text,
        hashed_password -> Text,
    }
}

diesel::joinable!(completions -> tests (test_id));
diesel::joinable!(tests -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    completions,
    tests,
    users,
);
