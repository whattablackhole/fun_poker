-- Adding new users
PREPARE insert_user (
    VARCHAR,
    VARCHAR,
    VARCHAR,
    priveleges_enum,
    VARCHAR
) AS
INSERT INTO
    "users" (
        "username",
        "password",
        "country",
        "type",
        "email"
    )
VALUES
    ($ 1, $ 2, $ 3, $ 4, $ 5);

EXECUTE insert_user(
    'Carter',
    'c8094bb1e0896a3f813036bdaeb37b753d9f4f5b',
    'USA',
    'admin',
    'carter_z@gmail.com'
);

EXECUTE insert_user(
    'Alice',
    'b8195cc2f1987b4f923147cbaeb48c86',
    'USA',
    'user',
    'alice_j@gmail.com'
);

EXECUTE insert_user(
    'Bob',
    'a8296dd3g2098c5g034258dcbec59d97',
    'USA',
    'admin',
    'bob_b@gmail.com'
);

EXECUTE insert_user(
    'David',
    'd8397ee4h3199d6h145369edcfd60e08',
    'USA',
    'staruser',
    'david_w@gmail.com'
);

EXECUTE insert_user(
    'Emma',
    'e8498ff5i4200e7i256470feddf71f19',
    'USA',
    'moderator',
    'emma_h@gmail.com'
);

EXECUTE insert_user(
    'Frank',
    'f8599gg6j5211f8j3675810feeg82g20',
    'USA',
    'user',
    'frank_m@gmail.com'
);

EXECUTE insert_user(
    'Grace',
    'g8600hh7k6222g9k4786921gfgh93h31',
    'USA',
    'moderator',
    'grace_l@gmail.com'
);

EXECUTE insert_user(
    'Henry',
    'h8701ii8l7233h0l5897032hghj04i42',
    'USA',
    'user',
    'henry_w@gmail.com'
);

EXECUTE insert_user(
    'Isla',
    'i8802jj9m8244i1m6908143ihik15j53',
    'USA',
    'moderator',
    'isla_y@gmail.com'
);

EXECUTE insert_user(
    'Jack',
    'j8903kk0n9255j2n7019254jijl26k64',
    'USA',
    'user',
    'jack_k@gmail.com'
);

-- Auth query
-- ARGUMENTS: username = Carter(which has unique constraint), password = hashed 64byte string
PREPARE select_user(VARCHAR, VARCHAR) AS
SELECT
    *
FROM
    "users"
WHERE
    "username" = $ 1
    AND "password" = $ 2;

EXECUTE select_user(
    'Carter',
    'c8094bb1e0896a3f813036bdaeb37b753d9f4f5b'
);

-- Creating new games by admin
PREPARE insert_game(
    INT,
    VARCHAR,
    game_type_enum,
    game_name_enum,
    INT
) AS
INSERT INTO
    "games"(
        "author_id",
        "title",
        "game_type",
        "game_name",
        "prize_pool"
    )
VALUES
    ($ 1, $ 2, $ 3, $ 4, $ 5);

EXECUTE insert_game(
    1,
    'SUNDAY TRILLION',
    'tournament',
    'holdem',
    10000
);

EXECUTE insert_game(
    1,
    'Free roll for everyone',
    'tournament',
    'holdem',
    10000
);

EXECUTE insert_game(
    1,
    'Bounty Hunter',
    'tournament',
    'holdem',
    10000
);

EXECUTE insert_game(1, 'Big', 'tournament', 'holdem', 10000);

-- When creating new games by user, we triggering the function which populates players_games table with id of game creator and game id
EXECUTE insert_game(2, 'Home game', 'tournament', 'holdem', 10000);

SELECT
    *
FROM
    players_games;

-- When user connected to game lobby, we add them to players_games relationship
PREPARE insert_players_games(INT, INT) AS
INSERT INTO
    "players_games"("user_id", "game_id")
VALUES
    ($ 1, $ 2);

EXECUTE insert_players_games(6, 1);

EXECUTE insert_players_games(7, 1);

EXECUTE insert_players_games(1, 1);

EXECUTE insert_players_games(2, 1);

EXECUTE insert_players_games(3, 1);

EXECUTE insert_players_games(4, 1);

EXECUTE insert_players_games(5, 1);

EXECUTE insert_players_games(6, 2);

EXECUTE insert_players_games(7, 2);

EXECUTE insert_players_games(1, 2);

EXECUTE insert_players_games(2, 2);

-- When user made an action in the game, we store it as history item in partitioned table;
PREPARE insert_game_history_item(
    INT,
    INT,
    player_action_enum,
    JSONB,
    JSONB,
    player_position_enum,
    NUMERIC
) AS
INSERT INTO
    "games_history_items"(
        "user_id",
        "game_id",
        "action",
        "cards",
        "street",
        "position",
        "amount"
    )
VALUES
    ($ 1, $ 2, $ 3, $ 4, $ 5, $ 6, $ 7);

-- Game with id 1  stored at games_history_items_1 partition
INSERT INTO
    "games_history_items"(
        "user_id",
        "game_id",
        "action",
        "amount",
        "cards",
        "street",
        "position"
    )
VALUES
    (
        6,
        1,
        'call',
        50,
        normalize_player_cards(
            '[{"rank": "8", "suit": "Spades"}, {"rank": "9", "suit": "Dimonds"}]' :: jsonb
        ),
        '[]' :: jsonb,
        'SB'
    ),
    (
        7,
        1,
        'call',
        100,
        normalize_player_cards(
            '[{"rank": "T", "suit": "Spades"}, {"rank": "J", "suit": "Dimonds"}]' :: jsonb
        ),
        '[]' :: jsonb,
        'BB'
    ),
    (
        1,
        1,
        'raise',
        300,
        normalize_player_cards(
            '[{"rank": "A", "suit": "Hearts"}, {"rank": "K", "suit": "Hearts"}]' :: jsonb
        ),
        '[]' :: jsonb,
        'MP'
    ),
    (
        2,
        1,
        'raise',
        900,
        normalize_player_cards(
            '[{"rank": "A", "suit": "Spades"}, {"rank": "A", "suit": "Dimonds"}]' :: jsonb
        ),
        '[]' :: jsonb,
        'MP+1'
    ),
    (
        3,
        1,
        'fold',
        0,
        normalize_player_cards(
            '[{"rank": "2", "suit": "Spades"}, {"rank": "3", "suit": "Dimonds"}]' :: jsonb
        ),
        '[]' :: jsonb,
        'MP+2'
    ),
    (
        4,
        1,
        'fold',
        0,
        normalize_player_cards(
            '[{"rank": "4", "suit": "Spades"}, {"rank": "5", "suit": "Dimonds"}]' :: jsonb
        ),
        '[]' :: jsonb,
        'CO'
    ),
    (
        5,
        1,
        'fold',
        0,
        normalize_player_cards(
            '[{"rank": "6", "suit": "Spades"}, {"rank": "7", "suit": "Dimonds"}]' :: jsonb
        ),
        '[]' :: jsonb,
        'BTN'
    ),
    (
        6,
        1,
        'fold',
        0,
        normalize_player_cards(
            '[{"rank": "8", "suit": "Spades"}, {"rank": "9", "suit": "Dimonds"}]' :: jsonb
        ),
        '[]' :: jsonb,
        'SB'
    ),
    (
        7,
        1,
        'fold',
        0,
        normalize_player_cards(
            '[{"rank": "T", "suit": "Spades"}, {"rank": "J", "suit": "Dimonds"}]' :: jsonb
        ),
        '[]' :: jsonb,
        'BB'
    ),
    (
        1,
        1,
        'call',
        600,
        normalize_player_cards(
            '[{"rank": "A", "suit": "Spades"}, {"rank": "A", "suit": "Dimonds"}]' :: jsonb
        ),
        '[]' :: jsonb,
        'MP'
    );

-- Game with id 2 stored at games_history_items_2 partition
INSERT INTO
    "games_history_items"(
        "user_id",
        "game_id",
        "action",
        "amount",
        "cards",
        "street",
        "position"
    )
VALUES
    (
        6,
        2,
        'call',
        50,
        normalize_player_cards(
            '[{"rank": "8", "suit": "Spades"}, {"rank": "9", "suit": "Dimonds"}]' :: jsonb
        ),
        '[]' :: jsonb,
        'SB'
    ),
    (
        7,
        2,
        'call',
        100,
        normalize_player_cards(
            '[{"rank": "T", "suit": "Spades"}, {"rank": "J", "suit": "Dimonds"}]' :: jsonb
        ),
        '[]' :: jsonb,
        'BB'
    ),
    (
        1,
        2,
        'raise',
        300,
        normalize_player_cards(
            '[{"rank": "A", "suit": "Hearts"}, {"rank": "K", "suit": "Hearts"}]' :: jsonb
        ),
        '[]' :: jsonb,
        'MP'
    ),
    (
        2,
        2,
        'raise',
        900,
        normalize_player_cards(
            '[{"rank": "A", "suit": "Spades"}, {"rank": "A", "suit": "Dimonds"}]' :: jsonb
        ),
        '[]' :: jsonb,
        'MP+1'
    );

-- Better to do above inserts with the prepared function:
-- EXECUTE insert_game_history_item(1, 1, 'raise', normalize_cards('[{"rank": "A", "suit": "Hearts"}, {"rank": "K", "suit": "Hearts"}]'::jsonb), '[]'::jsonb, 'UTG', 1000);
-- EXECUTE insert_game_history_item(2, 1, 'fold', normalize_cards('[{"rank": "2", "suit": "Spades"}, {"rank": "7", "suit": "Hearts"}]'::jsonb), '[]'::jsonb, 'UTG+1', 0);
-- Let imagine a game lobby window and it has a list with all player names and their countries. This query is capable to get all requried info to represent that list.
-- ARGUMENTS: game_id = 1
PREPARE select_user_list(INT) AS
SELECT
    "username",
    "country"
FROM
    "users"
WHERE
    "id" IN (
        SELECT
            "user_id"
        FROM
            "players_games"
        WHERE
            "game_id" = 1
    );

EXECUTE select_user_list(1);

-- Calling views to see some statistics;
SELECT
    *
FROM
    "registered_players_amount";

SELECT
    *
FROM
    "most_popular_hands_for_raise";

SELECT
    *
FROM
    "most_popular_hands_for_fold";

SELECT
    *
FROM
    "most_popular_hands_for_call";

SELECT
    *
FROM
    "most_popular_hands_for_raise_per_position";

SELECT
    *
FROM
    "most_popular_hands_for_call_per_position";

SELECT
    *
FROM
    "most_popular_hands_for_fold_per_position";

-- Adding user settings
-- TODO: use trigger in future
INSERT INTO
    "user_settings"("user_id", "show_banks_in_blinds")
VALUES
    (1, TRUE);

INSERT INTO
    "user_settings"("user_id", "show_banks_in_blinds")
VALUES
    (2, FALSE);

INSERT INTO
    "user_settings"("user_id", "show_banks_in_blinds", "language")
VALUES
    (3, TRUE, 'bel');

INSERT INTO
    "user_settings"("user_id", "show_banks_in_blinds")
VALUES
    (4, TRUE);

INSERT INTO
    "user_settings"("user_id", "show_banks_in_blinds", "language")
VALUES
    (5, FALSE, 'bel');

INSERT INTO
    "user_settings"("user_id", "show_banks_in_blinds")
VALUES
    (6, TRUE);

INSERT INTO
    "user_settings"("user_id")
VALUES
    (7);

INSERT INTO
    "user_settings"("user_id")
VALUES
    (8);

INSERT INTO
    "user_settings"("user_id")
VALUES
    (9);

INSERT INTO
    "user_settings"("user_id")
VALUES
    (10);

-- Updating user settings
PREPARE update_user_settings(INT, BOOLEAN) AS
UPDATE
    "user_settings"
SET
    "show_banks_in_blinds" = $ 2
WHERE
    "user_id" = $ 1;

EXECUTE update_user_settings(1, TRUE);

-- Getting users with their settings
SELECT
    *
FROM
    "users"
    JOIN "user_settings" ON "users"."id" = "user_settings"."user_id"
ORDER BY
    "id";