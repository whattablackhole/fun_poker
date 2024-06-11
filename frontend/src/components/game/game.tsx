import { useEffect, useState } from "react";
import { ClientState } from "../../types/client-state";
import PokerTable3d from "../poker_table_3d/poker-table-3d";
import ApiService from "../../services/api.service";
import BetHistory from "../../types/bet-history";
import { ActionType, Card, Player, PlayerPayload } from "../../types";
import GameStateService from "../../services/game-state.service";
import GameControls from "../game-controls/game-controls";
import "./game.css";


// data
// 1. my cards, my bank
// 2. player list
// 3. current state 
//    -- bank, player actions,
// -- update state acitons
// 1.init hands animation,
// 2.bet animation(banks),
// 3.fold animation
// 4.time bank animation


function Game() {
    // const canvasRef: RefObject<HTMLCanvasElement> = useRef(null);
    // const betInputRef: RefObject<HTMLInputElement> = useRef(null);
    const [gameState, setState] = useState<ClientState | undefined>(undefined);
    const [betHistory, setBetHistory] = useState<BetHistory>(new BetHistory());
    const [boardCards, setBoardCards] = useState<Card[] | undefined>();
    const [players, setPlayers] = useState<Player[]>();
    const [selfPlayer, setSelfPlayer] = useState<Player>();
    let prevStateCopy = gameState;

    const betClickHandler = (value: number, type: ActionType) => {
        let payload = PlayerPayload.create({ action: { actionType: type, bet: value }, lobbyId: gameState?.lobbyId, playerId: gameState?.playerId });

        // ApiService.sendMessage(payload);
    };

    

    useEffect(() => {
        // let subscription = ApiService.clientStateObserver.subscribe(async (newState: ClientState) => {
        //     await GameStateService.processNewState(newState, prevStateCopy, betHistory, setBoardCards, setBetHistory, setPlayers);
        //     setState(newState);
        //     let selfPlayer = newState.players.find(player => player.userId == newState.playerId)!;
        //     selfPlayer.cards = newState.cards;
        //     setSelfPlayer(selfPlayer);
        //     prevStateCopy = newState;
        // })


        // return () => {
        //     ApiService.clientStateObserver.unsubscribe(subscription);
        // }


    }, []);


    if (!gameState || !selfPlayer || !players) {
        return (
            <div>Game is not ready: Bad state</div>
        )
    }

    return (
        <div>
            <PokerTable3d selfPlayer={selfPlayer} players={players} betHistory={betHistory} buttonId={gameState.currButtonId} currPlayerId={gameState.currPlayerId} street={gameState.street} />
            <div className="game-controls">
                <GameControls selfPlayer={selfPlayer} gameState={gameState} betClickHandler={betClickHandler} />
            </div>

        </div>
    )
}




export default Game;