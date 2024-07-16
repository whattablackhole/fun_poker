import { useEffect, useState } from "react";
import "./game-controls.css";
import InputSlider from "./bet-slider";
import { Button, Grid } from "@mui/material";
import { ActionType } from "../../types/game_state";
import { ClientState } from "../../types/client_state";
import { Player } from "../../types/player";
import BetHistory from "../../types/bet-history";

function GameControls({
  gameState,
  player,
  betHistory,
  betClickHandler,
  spawnBotClickHandler,
}: {
  gameState: ClientState;
  player: Player;
  betHistory: BetHistory;
  betClickHandler: (value: number, type: ActionType) => void;
  spawnBotClickHandler: () => void;
}) {
  const [minRaiseValue, setMinRaiseValue] = useState(
    gameState.minAmountToRaise?.value ?? 0
  );

  const maxValue =
    player.bank +
    betHistory.getPlayerBetAmount(
      player.userId,
      gameState.street?.streetStatus
    );

  const [betSizeInputValue, setBetSizeInputValue] =
    useState<number>(minRaiseValue);

  useEffect(() => {
    if (gameState.minAmountToRaise?.value !== undefined) {
      if (player.bank < gameState.minAmountToRaise?.value) {
        setMinRaiseValue(maxValue);
      } else {
        setMinRaiseValue(gameState.minAmountToRaise?.value);
      }
    }
  }, [gameState.minAmountToRaise?.value]);

  useEffect(() => {
    setBetSizeInputValue(minRaiseValue);
  }, [minRaiseValue]);

  const handleBetSizeChange = (amount: number) => {
    setBetSizeInputValue(amount);
  };

  const minRaiseHandler = () => {
    setBetSizeInputValue(gameState.minAmountToRaise?.value ?? 0);
  };

  return (
    <Grid
      container
      alignItems="flex-end"
      flexDirection="column"
      sx={{ gap: "10px" }}
    >
      <Grid item sx={{ gap: "10px", display: "flex", flexDirection: "column" }}>
        <Button
          size="large"
          sx={{
            fontSize: "1.2rem",
            background: "linear-gradient(to bottom, lightgrey, darkgrey)",
            boxShadow: "0 0 0 1px black, 0 0 0 4px grey, 0 0 0 5px black",
            fontWeight: "800",
            width: "150px",
          }}
          className="control-button"
          onClick={() => spawnBotClickHandler()}
        >
          Spawn Bot
        </Button>
      </Grid>
      <Grid item sx={{ gap: "10px", display: "flex", flexDirection: "column" }}>
        <div style={{ display: "flex", justifyContent: "space-between" }}>
          <Button
            size="small"
            sx={{
              fontSize: "1rem",
              background: "linear-gradient(to bottom, lightgrey, darkgrey)",
              boxShadow: "0 0 0 1px black, 0 0 0 2px grey",
            }}
            className="control-button"
            onClick={() => minRaiseHandler()}
            disabled={
              gameState.currPlayerId?.value !== player.userId ||
              player.action?.actionType === ActionType.Fold ||
              minRaiseValue > player.bank
            }
          >
            Min
          </Button>
          <Button
            size="small"
            sx={{
              fontSize: "1rem",
              background: "linear-gradient(to bottom, lightgrey, darkgrey)",
              boxShadow: "0 0 0 1px black, 0 0 0 2px grey",
            }}
            className="control-button"
            disabled={
              gameState.currPlayerId?.value !== player.userId ||
              player.action?.actionType === ActionType.Fold ||
              50 > player.bank ||
              (gameState.amountToCall?.value ?? 0) > 50
            }
          >
            1/2
          </Button>
          <Button
            size="small"
            sx={{
              fontSize: "1rem",
              background: "linear-gradient(to bottom, lightgrey, darkgrey)",
              boxShadow: "0 0 0 1px black, 0 0 0 2px grey",
            }}
            className="control-button"
            // get pot size
            disabled={
              gameState.currPlayerId?.value !== player.userId ||
              player.action?.actionType === ActionType.Fold ||
              player.bank < 100
            }
          >
            Pot
          </Button>
          <Button
            size="small"
            sx={{
              fontSize: "1rem",
              background: "linear-gradient(to bottom, lightgrey, darkgrey)",
              boxShadow: "0 0 0 1px black, 0 0 0 2px grey",
            }}
            className="control-button"
            disabled={
              gameState?.currPlayerId?.value !== player.userId ||
              player.action?.actionType === ActionType.Fold ||
              player.bank === 0
            }
          >
            Max
          </Button>
        </div>
        <InputSlider
          value={betSizeInputValue}
          minValue={minRaiseValue}
          maxValue={maxValue}
          onValueChange={handleBetSizeChange}
          disable={!gameState.canRaise?.value}
        ></InputSlider>
      </Grid>
      <Grid item sx={{ gap: "20px", display: "flex" }}>
        <Button
          size="large"
          sx={{
            fontSize: "1.2rem",
            background: "linear-gradient(to bottom, lightgrey, darkgrey)",
            boxShadow: "0 0 0 1px black, 0 0 0 4px grey, 0 0 0 5px black",
            fontWeight: "800",
            width: "150px",
          }}
          className="fold_button control-button"
          disabled={
            gameState?.currPlayerId?.value !== player.userId ||
            player.action?.actionType === ActionType.Fold ||
            gameState.amountToCall?.value === 0
          }
          onClick={() => betClickHandler(0, ActionType.Fold)}
        >
          Fold
        </Button>
        <Button
          size="large"
          sx={{
            fontSize: "1.2rem",
            boxShadow: "0 0 0 1px black, 0 0 0 4px grey, 0 0 0 5px black",
            fontWeight: "800",
            background: "linear-gradient(to bottom, lightgrey, darkgrey)",
            width: "150px",
          }}
          className="call-check_button control-button"
          disabled={
            gameState?.currPlayerId?.value !== gameState.playerId ||
            player.action?.actionType === ActionType.Fold ||
            player.bank < (gameState.amountToCall?.value ?? 0)
          }
          onClick={() =>
            betClickHandler(
              betSizeInputValue,
              gameState.amountToCall?.value === 0
                ? ActionType.Check
                : ActionType.Call
            )
          }
        >
          {gameState.amountToCall?.value === 0 ? "Check" : "Call"}
        </Button>
        <Button
          size="large"
          sx={{
            fontSize: "1.2rem",
            boxShadow: "0 0 0 1px black, 0 0 0 4px grey, 0 0 0 5px black",
            fontWeight: "800",
            background: "linear-gradient(to bottom, lightgrey, darkgrey)",
            width: "150px",
          }}
          className="raise_button control-button"
          disabled={
            gameState?.currPlayerId?.value !== gameState.playerId ||
            player.action?.actionType === ActionType.Fold ||
            player.bank === 0 ||
            !gameState.canRaise?.value
            // maxValue === minRaiseValue
          }
          onClick={() => betClickHandler(betSizeInputValue, ActionType.Raise)}
        >
          Raise
        </Button>
        {/* </ButtonGroup> */}
      </Grid>
    </Grid>
  );
}

export default GameControls;
