// @generated automatically by Diesel CLI.

diesel::table! {
    auth_group (id) {
        id -> Int4,
        name -> Varchar,
    }
}

diesel::table! {
    auth_group_permissions (id) {
        id -> Int4,
        group_id -> Int4,
        permission_id -> Int4,
    }
}

diesel::table! {
    auth_permission (id) {
        id -> Int4,
        name -> Varchar,
        content_type_id -> Int4,
        codename -> Varchar,
    }
}

diesel::table! {
    award (id) {
        id -> Int4,
        name -> Varchar,
        description -> Text,
        groupName -> Varchar,
        requisition -> Text,
    }
}

diesel::table! {
    award_application (id) {
        id -> Int4,
        status -> Int4,
        comment -> Text,
        created -> Timestamptz,
        reviewed -> Nullable<Timestamptz>,
        applier_id -> Int4,
        award_id -> Int4,
        reviewer_id -> Nullable<Int4>,
        reason -> Text,
    }
}

diesel::table! {
    bookmark (id) {
        id -> Int4,
        value -> Int2,
        puzzle_id -> Int4,
        user_id -> Int4,
    }
}

diesel::table! {
    chatmessage (id) {
        id -> Int4,
        content -> Text,
        created -> Nullable<Timestamptz>,
        editTimes -> Int4,
        chatroom_id -> Int4,
        user_id -> Int4,
        modified -> Timestamptz,
    }
}

diesel::table! {
    chatroom (id) {
        id -> Int4,
        name -> Varchar,
        description -> Text,
        created -> Date,
        user_id -> Int4,
        official -> Bool,
        public -> Bool,
    }
}

diesel::table! {
    comment (id) {
        id -> Int4,
        content -> Text,
        spoiler -> Bool,
        puzzle_id -> Int4,
        user_id -> Int4,
    }
}

diesel::table! {
    dialogue (id) {
        id -> Int4,
        question -> Text,
        answer -> Text,
        good -> Bool,
        #[sql_name = "true"]
        true_ -> Bool,
        created -> Timestamptz,
        answeredtime -> Nullable<Timestamptz>,
        puzzle_id -> Int4,
        user_id -> Int4,
        answerEditTimes -> Int4,
        questionEditTimes -> Int4,
        qno -> Int4,
        modified -> Timestamptz,
    }
}

diesel::table! {
    direct_message (id) {
        id -> Int4,
        content -> Text,
        created -> Timestamptz,
        receiver_id -> Int4,
        sender_id -> Int4,
        editTimes -> Int4,
        modified -> Timestamptz,
    }
}

diesel::table! {
    django_admin_log (id) {
        id -> Int4,
        action_time -> Timestamptz,
        object_id -> Nullable<Text>,
        object_repr -> Varchar,
        action_flag -> Int2,
        change_message -> Text,
        content_type_id -> Nullable<Int4>,
        user_id -> Int4,
    }
}

diesel::table! {
    django_content_type (id) {
        id -> Int4,
        app_label -> Varchar,
        model -> Varchar,
    }
}

diesel::table! {
    django_migrations (id) {
        id -> Int4,
        app -> Varchar,
        name -> Varchar,
        applied -> Timestamptz,
    }
}

diesel::table! {
    django_session (session_key) {
        session_key -> Varchar,
        session_data -> Text,
        expire_date -> Timestamptz,
    }
}

diesel::table! {
    dm_read (id) {
        id -> Int4,
        user_id -> Int4,
        with_user_id -> Int4,
        dm_id -> Int4,
    }
}

diesel::table! {
    event (id) {
        id -> Int4,
        title -> Varchar,
        banner_img_url -> Varchar,
        status -> Int4,
        start_time -> Timestamptz,
        end_time -> Timestamptz,
        page_link -> Varchar,
        page_src -> Text,
        user_id -> Int4,
    }
}

diesel::table! {
    event_award (id) {
        id -> Int4,
        award_id -> Int4,
        event_id -> Int4,
    }
}

diesel::table! {
    favorite_chatroom (id) {
        id -> Int4,
        chatroom_id -> Int4,
        user_id -> Int4,
    }
}

diesel::table! {
    hasura_direct_message_group_trigger (user_id) {
        user_id -> Int4,
        last_dm_id -> Int4,
    }
}

diesel::table! {
    hasura_int_groupby_trigger (group) {
        group -> Int4,
        value -> Int8,
    }
}

diesel::table! {
    hasura_user_ranking_trigger (user_id) {
        user_id -> Int4,
        value -> Int8,
    }
}

diesel::table! {
    hint (id) {
        id -> Int4,
        content -> Text,
        created -> Timestamptz,
        puzzle_id -> Int4,
        edittimes -> Int4,
        receiver_id -> Nullable<Int4>,
        modified -> Timestamptz,
    }
}

diesel::table! {
    image (id) {
        id -> Uuid,
        user_id -> Int4,
        puzzle_id -> Nullable<Int4>,
        created -> Timestamptz,
        content_type -> Varchar,
    }
}

diesel::table! {
    license (id) {
        id -> Int4,
        user_id -> Nullable<Int4>,
        name -> Varchar,
        description -> Text,
        url -> Nullable<Varchar>,
        contract -> Nullable<Text>,
    }
}

diesel::table! {
    puzzle (id) {
        id -> Int4,
        title -> Varchar,
        yami -> Int4,
        genre -> Int4,
        content -> Text,
        solution -> Text,
        created -> Timestamptz,
        modified -> Timestamptz,
        status -> Int4,
        memo -> Text,
        user_id -> Int4,
        anonymous -> Bool,
        dazed_on -> Date,
        grotesque -> Bool,
        license_id -> Nullable<Int4>,
    }
}

diesel::table! {
    puzzle_tag (id) {
        id -> Int4,
        puzzle_id -> Int4,
        tag_id -> Int4,
        user_id -> Int4,
    }
}

diesel::table! {
    replay (id) {
        id -> Int4,
        title -> Varchar,
        milestones -> Jsonb,
        puzzle_id -> Nullable<Int4>,
        user_id -> Int4,
        created -> Timestamptz,
    }
}

diesel::table! {
    replay_dialogue (id) {
        id -> Int4,
        replay_id -> Int4,
        question -> Text,
        answer -> Text,
        good -> Bool,
        #[sql_name = "true"]
        true_ -> Bool,
        keywords -> Jsonb,
        milestones -> Jsonb,
        dependency -> Text,
    }
}

diesel::table! {
    schedule (id) {
        id -> Int4,
        content -> Text,
        created -> Timestamptz,
        scheduled -> Timestamptz,
        user_id -> Int4,
    }
}

diesel::table! {
    star (id) {
        id -> Int4,
        value -> Int2,
        puzzle_id -> Int4,
        user_id -> Int4,
    }
}

diesel::table! {
    sui_hei_puzzle_tokenize_cache (id) {
        id -> Int4,
        puzzle_id -> Int4,
        tokens -> Jsonb,
    }
}

diesel::table! {
    sui_hei_user_groups (id) {
        id -> Int4,
        user_id -> Int4,
        group_id -> Int4,
    }
}

diesel::table! {
    sui_hei_user_user_permissions (id) {
        id -> Int4,
        user_id -> Int4,
        permission_id -> Int4,
    }
}

diesel::table! {
    tag (id) {
        id -> Int4,
        name -> Varchar,
        created -> Timestamptz,
    }
}

diesel::table! {
    user (id) {
        id -> Int4,
        password -> Varchar,
        last_login -> Nullable<Timestamptz>,
        is_superuser -> Bool,
        username -> Varchar,
        first_name -> Varchar,
        last_name -> Varchar,
        email -> Varchar,
        is_staff -> Bool,
        is_active -> Bool,
        date_joined -> Timestamptz,
        nickname -> Varchar,
        profile -> Text,
        current_award_id -> Nullable<Int4>,
        hide_bookmark -> Bool,
        icon -> Nullable<Varchar>,
        default_license_id -> Nullable<Int4>,
    }
}

diesel::table! {
    user_award (id) {
        id -> Int4,
        created -> Date,
        award_id -> Int4,
        user_id -> Int4,
    }
}

diesel::joinable!(auth_group_permissions -> auth_group (group_id));
diesel::joinable!(auth_group_permissions -> auth_permission (permission_id));
diesel::joinable!(auth_permission -> django_content_type (content_type_id));
diesel::joinable!(award_application -> award (award_id));
diesel::joinable!(bookmark -> puzzle (puzzle_id));
diesel::joinable!(bookmark -> user (user_id));
diesel::joinable!(chatmessage -> chatroom (chatroom_id));
diesel::joinable!(chatmessage -> user (user_id));
diesel::joinable!(chatroom -> user (user_id));
diesel::joinable!(comment -> puzzle (puzzle_id));
diesel::joinable!(comment -> user (user_id));
diesel::joinable!(dialogue -> puzzle (puzzle_id));
diesel::joinable!(dialogue -> user (user_id));
diesel::joinable!(django_admin_log -> django_content_type (content_type_id));
diesel::joinable!(django_admin_log -> user (user_id));
diesel::joinable!(dm_read -> direct_message (dm_id));
diesel::joinable!(event -> user (user_id));
diesel::joinable!(event_award -> award (award_id));
diesel::joinable!(event_award -> event (event_id));
diesel::joinable!(favorite_chatroom -> chatroom (chatroom_id));
diesel::joinable!(favorite_chatroom -> user (user_id));
diesel::joinable!(hasura_direct_message_group_trigger -> user (user_id));
diesel::joinable!(hasura_user_ranking_trigger -> user (user_id));
diesel::joinable!(hint -> puzzle (puzzle_id));
diesel::joinable!(hint -> user (receiver_id));
diesel::joinable!(image -> puzzle (puzzle_id));
diesel::joinable!(image -> user (user_id));
diesel::joinable!(puzzle -> license (license_id));
diesel::joinable!(puzzle -> user (user_id));
diesel::joinable!(puzzle_tag -> puzzle (puzzle_id));
diesel::joinable!(puzzle_tag -> tag (tag_id));
diesel::joinable!(puzzle_tag -> user (user_id));
diesel::joinable!(replay -> puzzle (puzzle_id));
diesel::joinable!(replay -> user (user_id));
diesel::joinable!(replay_dialogue -> replay (replay_id));
diesel::joinable!(schedule -> user (user_id));
diesel::joinable!(star -> puzzle (puzzle_id));
diesel::joinable!(star -> user (user_id));
diesel::joinable!(sui_hei_puzzle_tokenize_cache -> puzzle (puzzle_id));
diesel::joinable!(sui_hei_user_groups -> auth_group (group_id));
diesel::joinable!(sui_hei_user_groups -> user (user_id));
diesel::joinable!(sui_hei_user_user_permissions -> auth_permission (permission_id));
diesel::joinable!(sui_hei_user_user_permissions -> user (user_id));
diesel::joinable!(user_award -> award (award_id));

diesel::allow_tables_to_appear_in_same_query!(
    auth_group,
    auth_group_permissions,
    auth_permission,
    award,
    award_application,
    bookmark,
    chatmessage,
    chatroom,
    comment,
    dialogue,
    direct_message,
    django_admin_log,
    django_content_type,
    django_migrations,
    django_session,
    dm_read,
    event,
    event_award,
    favorite_chatroom,
    hasura_direct_message_group_trigger,
    hasura_int_groupby_trigger,
    hasura_user_ranking_trigger,
    hint,
    image,
    license,
    puzzle,
    puzzle_tag,
    replay,
    replay_dialogue,
    schedule,
    star,
    sui_hei_puzzle_tokenize_cache,
    sui_hei_user_groups,
    sui_hei_user_user_permissions,
    tag,
    user,
    user_award,
);
