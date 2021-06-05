table! {
    tag_aggr (id) {
        id -> Int4,
        name -> Varchar,
        created -> Timestamptz,
        puzzle_tag_count -> Int8,
    }
}
