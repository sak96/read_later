diesel::table! {
    articles (id) {
        id -> Integer,
        title -> Text,
        body -> Text,
        created_at -> Text,
    }
}

diesel::table! {
    settings (name) {
        name -> Text,
        value -> Text,
        default_value -> Text,
    }
}
