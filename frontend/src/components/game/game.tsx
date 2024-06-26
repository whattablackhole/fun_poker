import { useEffect, useRef, useState } from "react";
import PokerTable3d from "../poker_table_3d/poker-table-3d";
import ApiService from "../../services/api.service";
import BetHistory from "../../types/bet-history";
import { ActionType, Card, Player, PlayerPayload } from "../../types";
import GameStateService from "../../services/game-state.service";
import GameControls from "../game-controls/game-controls";
import "./game.css";
import { useWebSocket } from "../../providers/web-socket-provider";
import { ResponseMessageType } from "../../types/responses";
import { ClientState } from "../../types/client_state";
import { SpawnBotRequest } from "../../types/requests";
import { BotModel } from "../../types/ai_bot_player";

function Game() {
    // const canvasRef: RefObject<HTMLCanvasElement> = useRef(null);
    // const betInputRef: RefObject<HTMLInputElement> = useRef(null);
    const [gameState, setState] = useState<ClientState | undefined>(undefined);
    const [betHistory, setBetHistory] = useState<BetHistory>(new BetHistory());
    const [boardCards, setBoardCards] = useState<Card[] | undefined>();
    const [players, setPlayers] = useState<Player[]>();
    const selfPlayer = players?.find((p) => p.userId === gameState?.playerId)!;
    let prevStateCopy = gameState;
    const queueRef = useRef(Promise.resolve());

    let { addEventListener, removeEventListener, connection } = useWebSocket();

    const stateUpdateHandler = async (state: ClientState) => {
        queueRef.current = queueRef.current.then(async () => {
          const newState = await GameStateService.processNewState(state, prevStateCopy, betHistory, setBoardCards, setBetHistory, setPlayers);
          prevStateCopy = newState;
          setState(newState);
        });
        return queueRef.current;
      };

    useEffect(() => {
        addEventListener(ResponseMessageType.ClientState.toString(), stateUpdateHandler);

        return () => {
            removeEventListener(ResponseMessageType.ClientState.toString(), stateUpdateHandler);
        }
    }, []);

    const spawnBotClickHandler = () => {
        let payload = SpawnBotRequest.create({ lobbyId: gameState?.lobbyId,model: BotModel.Llama3_70b_8192 });
        ApiService.spawnBot(payload);
    };

    const betClickHandler = (value: number, type: ActionType) => {
        let payload = PlayerPayload.create({ action: { actionType: type, bet: value, playerId: selfPlayer.userId }, lobbyId: gameState?.lobbyId, playerId: selfPlayer.userId });
        connection?.current?.send(PlayerPayload.toBinary(payload));
    };

    if (!gameState || !players) {
        return (
            <div>Game is not ready: Bad state</div>
        )
    }

    return (
        <div>
            <PokerTable3d players={players} gameStatus={gameState.gameStatus} betHistory={betHistory} buttonId={gameState.currButtonId?.value} currPlayerId={gameState.currPlayerId?.value} street={gameState.street} />
            <div className="game-controls">
                <GameControls gameState={gameState} player={selfPlayer} betClickHandler={betClickHandler} spawnBotClickHandler={spawnBotClickHandler} />
            </div>

        </div>
    )
}




export default Game;