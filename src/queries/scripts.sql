CREATE OR REPLACE FUNCTION insert_into_players_lobbies()
RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO players_lobbies(lobby_id, player_id)
    VALUES (NEW.id, NEW.author_id);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;


CREATE TRIGGER insert_players_lobbies_trigger
AFTER INSERT ON lobbies
FOR EACH ROW
EXECUTE FUNCTION insert_into_players_lobbies();