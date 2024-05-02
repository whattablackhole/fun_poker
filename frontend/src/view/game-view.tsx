import { useLocation } from "react-router-dom";
import PokerTable from "../components/poker_table/poker-table.tsx";

function GameView() {
    const location = useLocation();
    const state = location.state;
    return PokerTable(state);
}

export default GameView