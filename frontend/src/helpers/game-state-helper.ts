import { Player } from "../types";



class GameStateProcessHelper {


    static center_players_by_self(players: Player[], myselfId: number): Player[] {
        let index = this.get_index_by_player_id(players, myselfId);

        if (index !== -1) {
            return [...players.slice(index), ...players.slice(0, index)];

        } else {
            return [];
        }
    }

    static get_index_by_player_id(players: Player[], id: number): number {
        return players.findIndex((p) => p.userId == id)
    }




}
export default GameStateProcessHelper;

