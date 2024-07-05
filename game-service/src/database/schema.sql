DO $ $ BEGIN IF NOT EXISTS (
    SELECT
        1
    FROM
        pg_type
    WHERE
        typname = 'game_name_enum'
) THEN CREATE TYPE game_name_enum AS ENUM('holdem');

END IF;

IF NOT EXISTS (
    SELECT
        1
    FROM
        pg_type
    WHERE
        typname = 'support_money_currency_enum'
) THEN CREATE TYPE support_money_currency_enum AS ENUM('usd');

END IF;

IF NOT EXISTS (
    SELECT
        1
    FROM
        pg_type
    WHERE
        typname = 'game_type_enum'
) THEN CREATE TYPE game_type_enum AS ENUM('tournament', 'cash');

END IF;

IF NOT EXISTS (
    SELECT
        1
    FROM
        pg_type
    WHERE
        typname = 'game_currency_enum'
) THEN CREATE TYPE game_currency_enum AS ENUM('virtual_chips', 'money');

END IF;

IF NOT EXISTS (
    SELECT
        1
    FROM
        pg_type
    WHERE
        typname = 'priveleges_enum'
) THEN CREATE TYPE priveleges_enum AS ENUM('admin', 'user', 'staruser', 'moderator');

END IF;

IF NOT EXISTS (
    SELECT
        1
    FROM
        pg_type
    WHERE
        typname = 'theme_enum'
) THEN CREATE TYPE theme_enum AS ENUM('primary', 'custom');

END IF;

IF NOT EXISTS (
    SELECT
        1
    FROM
        pg_type
    WHERE
        typname = 'player_action_enum'
) THEN CREATE TYPE player_action_enum AS ENUM('call', 'raise', 'fold');

END IF;

IF NOT EXISTS (
    SELECT
        1
    FROM
        pg_type
    WHERE
        typname = 'language_enum'
) THEN CREATE TYPE language_enum AS ENUM ('en', 'bel');

END IF;

IF NOT EXISTS (
    SELECT
        1
    FROM
        pg_type
    WHERE
        typname = 'player_position_enum'
) THEN CREATE TYPE player_position_enum AS ENUM (
    'UTG',
    -- Under the Gun
    'UTG+1',
    -- Under the Gun +1
    'UTG+2',
    -- Under the Gun +2
    'MP',
    -- Middle Position
    'MP+1',
    -- Middle Position +1
    'MP+2',
    -- Middle Position +2
    'CO',
    -- Cutoff
    'BTN',
    -- Button
    'SB',
    -- Small Blind
    'BB' -- Big Blind
);

END IF;

END $ $;

-- ***** INIT Validators *****
-- *******************************************************************************************************************************************************************************
-- Create extended(for readability) validating function for constrait email field
CREATE
OR REPLACE FUNCTION is_valid_email (email_address TEXT) RETURNS BOOLEAN AS $ $ BEGIN RETURN email_address ~ '^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$';

END;

$ $ LANGUAGE plpgsql;

-- ***** INIT TABLES *****
-- *******************************************************************************************************************************************************************************
CREATE TABLE IF NOT EXISTS "app_settings" (
    "id" SERIAL PRIMARY KEY,
    "money_currency" support_money_currency_enum NOT NULL DEFAULT ('usd'),
    "currenty_type" game_currency_enum NOT NULL DEFAULT ('virtual_chips')
);

CREATE TABLE IF NOT EXISTS "users" (
    "id" SERIAL PRIMARY KEY,
    "username" VARCHAR(32) NOT NULL UNIQUE,
    "password" VARCHAR(64) NOT NULL,
    "country" VARCHAR(32) NOT NULL,
    "balance" NUMERIC(12, 2) NOT NULL DEFAULT (0) CHECK ("balance" >= 0),
    "type" priveleges_enum NOT NULL DEFAULT ('user'),
    "email" VARCHAR(100) CHECK (is_valid_email ("email"))
);

-- Settings for a particular user:
-- If you played poker online, you might noticed that it's very handy to see players' stacks(banks) in blinds
-- other settings are described by their names
CREATE TABLE IF NOT EXISTS "user_settings" (
    "user_id" SERIAL PRIMARY KEY,
    FOREIGN KEY ("user_id") REFERENCES "users" ("id"),
    "language" language_enum NOT NULL DEFAULT ('en'),
    "show_banks_in_blinds" BOOLEAN DEFAULT (FALSE),
    "theme" theme_enum NOT NULL DEFAULT ('primary')
);

CREATE TABLE IF NOT EXISTS "games" (
    "id" SERIAL PRIMARY KEY,
    "author_id" INT,
    "title" VARCHAR(32) NOT NULL,
    "game_name" game_name_enum NOT NULL,
    "game_type" game_type_enum NOT NULL,
    "prize_pool" INT NOT NULL,
    "started" BOOLEAN NOT NULL DEFAULT (FALSE),
    "created_date" TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT (NOW()),
    FOREIGN KEY ("author_id") REFERENCES "users" ("id")
);

-- Create relationship between user as player with a game. It might be usefull to know what players are registered in a particular game
CREATE TABLE IF NOT EXISTS "players_games" (
    "user_id" INT,
    "game_id" INT,
    FOREIGN KEY ("user_id") REFERENCES "users" ("id"),
    FOREIGN KEY ("game_id") REFERENCES "games" ("id"),
    -- we have to ensure that player wont enter the same game twice
    PRIMARY KEY ("user_id", "game_id")
);

-- Here I process storing player actions history e.g. raise with ace+king suited up to 20(amount) on river(street) or fold with NULL(0) etc...
CREATE TABLE IF NOT EXISTS "games_history_items" (
    "id" SERIAL,
    "user_id" INT,
    "game_id" INT,
    "action" player_action_enum NOT NULL,
    "cards" JSONB NOT NULL,
    "street" JSONB NOT NULL,
    "position" player_position_enum NOT NULL,
    "amount" NUMERIC(12, 2) NOT NULL,
    PRIMARY KEY("id", "game_id"),
    FOREIGN KEY ("game_id") REFERENCES "games" ("id"),
    FOREIGN KEY ("user_id") REFERENCES "users" ("id") -- TODO: add player_current_bank and other info
) PARTITION BY LIST (game_id);

-- ***** INIT VIEWS *****
-- *******************************************************************************************************************************************************************************
-- Here I defined several views representing some useful statistics
CREATE
OR REPLACE VIEW "registered_players_amount" AS
SELECT
    "game_id",
    COUNT(*) AS "total"
FROM
    "players_games"
GROUP BY
    "game_id";

CREATE
OR REPLACE VIEW "most_popular_hands_for_raise" AS
SELECT
    "cards",
    COUNT("cards") AS "total of raises"
FROM
    "games_history_items"
WHERE
    "action" = 'raise'
GROUP BY
    "cards"
ORDER BY
    "total of raises" DESC;

CREATE
OR REPLACE VIEW "most_popular_hands_for_fold" AS
SELECT
    "cards",
    COUNT("cards") AS "total of folds"
FROM
    "games_history_items"
WHERE
    "action" = 'fold'
GROUP BY
    "cards"
ORDER BY
    "total of folds" DESC;

CREATE
OR REPLACE VIEW "most_popular_hands_for_call" AS
SELECT
    "cards",
    COUNT("cards") AS "total of calls"
FROM
    "games_history_items"
WHERE
    "action" = 'call'
GROUP BY
    "cards"
ORDER BY
    "total of calls" DESC;

CREATE
OR REPLACE VIEW "most_popular_hands_for_raise_per_position" AS
SELECT
    "cards",
    "position",
    COUNT("cards") AS "total of raises"
FROM
    "games_history_items"
WHERE
    "action" = 'raise'
GROUP BY
    "cards",
    "position"
ORDER BY
    "total of raises" DESC,
    "position";

CREATE
OR REPLACE VIEW "most_popular_hands_for_call_per_position" AS
SELECT
    "cards",
    "position",
    COUNT("cards") AS "total of calls"
FROM
    "games_history_items"
WHERE
    "action" = 'call'
GROUP BY
    "cards",
    "position"
ORDER BY
    "total of calls" DESC,
    "position";

CREATE
OR REPLACE VIEW "most_popular_hands_for_fold_per_position" AS
SELECT
    "cards",
    "position",
    COUNT("cards") AS "total of folds"
FROM
    "games_history_items"
WHERE
    "action" = 'fold'
GROUP BY
    "cards",
    "position"
ORDER BY
    "total of folds" DESC,
    "position";

-- ***** INIT FUNCTIONS AND TRIGGERS *****
-- *******************************************************************************************************************************************************************************
-- When user enters a tournament or other game, i need to create relation between user and game
-- I assume that my app gives ability to users to create their own games
-- and when someone creates a new game, we have to add this user to players_games table if user is not admin or moderator
CREATE
OR REPLACE FUNCTION insert_into_players_games () RETURNS TRIGGER AS $ $ BEGIN IF (
    SELECT
        "type"
    FROM
        "users"
    WHERE
        "id" = NEW.author_id
) NOT IN ('admin', 'moderator') THEN
INSERT INTO
    "players_games"("game_id", "user_id")
VALUES
    (NEW.id, NEW.author_id);

END IF;

RETURN NULL;

END;

$ $ LANGUAGE plpgsql;

-- Lets assume my app could have 100 tournaments with 100 players playing simultaneously.
-- In such case games_history will be increased very fast as it stores each players' action for each game.
-- That's why I decided to partition my game_history table on game_id so that i could move its processing to other server(I assume it's possible) or atleast make it asynchronous
CREATE
OR REPLACE FUNCTION create_partition_trigger_function() RETURNS TRIGGER AS $ $ BEGIN IF NOT EXISTS (
    SELECT
        1
    FROM
        information_schema.tables
    WHERE
        table_name = 'games_history_items_' || NEW.id
) THEN EXECUTE format(
    'CREATE TABLE games_history_items_%s PARTITION OF games_history_items FOR VALUES IN (%s);',
    NEW.id,
    NEW.id
);

END IF;

RETURN NULL;

END;

$ $ LANGUAGE plpgsql;

-- Create triggers to invoke the trigger functions on INSERT
CREATE TRIGGER create_partition_trigger
AFTER
INSERT
    ON games FOR EACH ROW EXECUTE FUNCTION create_partition_trigger_function();

CREATE TRIGGER insert_into_players_games_trigger
AFTER
INSERT
    ON games FOR EACH ROW EXECUTE FUNCTION insert_into_players_games();

-- To be sure that our array is sorted before to insert into the table, i use built in jsonb_agg function to map values ordered by rank and suit
-- to make sure that we can properly aggregate statistics
-- For example, if i store this [{"rank": "A", "suit": "Hearts"}, {"rank": "K", "suit": "Hearts"}] and add one more time the same array but with swapped items
-- [{"rank": "K", "suit": "Hearts"}, {"rank": "A", "suit": "Hearts"}] "GROUP BY" would create two groups instead of a single one
CREATE
OR REPLACE FUNCTION normalize_player_cards (cards JSONB) RETURNS JSONB AS $ $ BEGIN RETURN (
    SELECT
        jsonb_agg(card)
    FROM
        (
            SELECT
                card
            FROM
                jsonb_array_elements(cards) AS card
            ORDER BY
                card ->> 'rank',
                card ->> 'suit'
        ) AS sorted_cards
);

END;

$ $ LANGUAGE plpgsql;

-- ***** INIT INDEXES *****
-- *******************************************************************************************************************************************************************************
CREATE INDEX "search_history_item_by_action" ON "games_history_items"("action");