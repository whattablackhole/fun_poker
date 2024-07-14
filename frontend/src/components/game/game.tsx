import { useEffect, useRef, useState } from "react";
import PokerTable3d from "../poker_table_3d/poker-table-3d";
import ApiService from "../../services/api.service";
import BetHistory from "../../types/bet-history";
import GameStateService from "../../services/game-state.service";
import GameControls from "../game-controls/game-controls";
import { useWebSocketContext } from "../../contexts/websocket-context";
import { ResponseMessageType } from "../../types/responses";
import { ClientState } from "../../types/client_state";
import { PlayerActionRequest, SpawnBotRequest } from "../../types/requests";
import { BotModel } from "../../types/ai_bot_player";
import { ActionType } from "../../types/game_state";
import useQuery from "../../hooks/useQuery";
import { useNavigate } from "react-router-dom";
import "./game.css";
import { Card } from "../../types/card";
import { Player } from "../../types/player";
import { CircularProgress, Typography } from "@mui/material";

const wsUrl = import.meta.env.VITE_WS_URL;

function Game() {
  const query = useQuery();
  const navigate = useNavigate();

  let websocketService = useWebSocketContext();

  let [loading, setLoading] = useState(true);

  let onConnectionClose = () => {
    navigate("/");
  };

  useEffect(() => {
    const lobbyId = query.get("lobby_id");

    if (lobbyId) {
      websocketService
        .connect(`${wsUrl}/join_lobby?lobby_id=${lobbyId}`)
        .then(() => {
          websocketService.addEventListener(
            ResponseMessageType.ClientState.toString(),
            stateUpdateHandler
          );
          setLoading(false);
        })
        .catch(() => {
          navigate("/");
        });
        websocketService.addEventListener(CloseEvent.name, onConnectionClose);

      return () => {
        websocketService.disconnect();
      };
    } else {
      navigate("/");
    }
  }, []);

  const [gameState, setState] = useState<ClientState | undefined>(undefined);
  const [betHistory, setBetHistory] = useState<BetHistory>(new BetHistory());
  const [boardCards, setBoardCards] = useState<Card[] | undefined>();
  const [players, setPlayers] = useState<Player[]>();
  const selfPlayer = players?.find((p) => p.userId === gameState?.playerId)!;
  let prevStateCopy = gameState;
  const queueRef = useRef(Promise.resolve());

  const stateUpdateHandler = async (state: ClientState) => {
    console.log(state);
    queueRef.current = queueRef.current.then(async () => {
      const newState = await GameStateService.processNewState(
        state,
        prevStateCopy,
        betHistory,
        setBoardCards,
        setBetHistory,
        setPlayers
      );
      prevStateCopy = newState;
      setState(newState);
    });
    return queueRef.current;
  };

  const spawnBotClickHandler = () => {
    let payload = SpawnBotRequest.create({
      lobbyId: gameState?.lobbyId,
      model: BotModel.Llama3_70b_8192,
    });
    ApiService.spawnBot(payload);
  };

  const betClickHandler = (value: number, type: ActionType) => {
    let payload = PlayerActionRequest.create({
      action: { actionType: type, bet: value, playerId: selfPlayer.userId },
      lobbyId: gameState?.lobbyId,
      playerId: selfPlayer.userId,
    });
    websocketService.sendMessage(PlayerActionRequest.toBinary(payload));
  };

  if (loading) {
    return (
      <div className="game-connecting">
        <CircularProgress variant="indeterminate" thickness={5} size={100} />
        <Typography variant="h4">Connecting...</Typography>
      </div>
    );
  }

  if (!gameState || !players) {
    return (
      <div className="game-connecting">
        <CircularProgress variant="indeterminate" thickness={5} size={100} />
        <Typography variant="h4">Loading game state...</Typography>
      </div>
    );
  }

  return (
    <div>
      <PokerTable3d
        players={players}
        gameStatus={gameState.gameStatus}
        betHistory={betHistory}
        buttonId={gameState.currButtonId?.value}
        currPlayerId={gameState.currPlayerId?.value}
        street={gameState.street}
      />
      <div className="game-controls">
        <GameControls
          gameState={gameState}
          player={selfPlayer}
          betClickHandler={betClickHandler}
          spawnBotClickHandler={spawnBotClickHandler}
        />
      </div>
    </div>
  );
}

export default Game;
