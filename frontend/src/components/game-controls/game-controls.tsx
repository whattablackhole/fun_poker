import { useState } from "react";
import { ActionType, ClientState, Player } from "../../types";
import "./game-controls.css";
import InputSlider from "./bet-slider";
import { Button, Grid } from "@mui/material";

function GameControls({
  gameState,
  player,
  betClickHandler,
  spawnBotClickHandler,
}: {
  gameState: ClientState;
  player: Player;
  betClickHandler: (value: number, type: ActionType) => void;
  spawnBotClickHandler: () => void;
}) {
  const [betSizeInputValue, setBetSizeInputValue] = useState<number>(0);

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
            // get min size size
            disabled={
              gameState?.currPlayerId?.value !== player.userId ||
              player.action?.actionType === ActionType.Fold ||
              50 > player.bank ||
              (gameState.amountToCall?.value ?? 0) > 50
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
            // get half blind size
            disabled={
              gameState?.currPlayerId?.value !== player.userId ||
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
              gameState?.currPlayerId?.value !== player.userId ||
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
          defaultValue={gameState.amountToCall?.value ?? 0}
          maxValue={player.bank}
          onValueChange={setBetSizeInputValue}
        ></InputSlider>
      </Grid>
      <Grid item sx={{ gap: "20px", display: "flex" }}>
        {/* <ButtonGroup variant="outlined" aria-label="betting options" sx={{
                        // backgroundColor: 'green',
                        // borderRadius: '8px',
                        // border: '2px solid black',
                        gap: '20px'
                    }}> */}
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
            player.action?.actionType === ActionType.Fold
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
          onClick={() => betClickHandler(betSizeInputValue, ActionType.Call)}
        >
          Call
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
            player.bank < (gameState.minAmountToRaise?.value ?? 0) ||
            betSizeInputValue < (gameState.minAmountToRaise?.value ?? 0)
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
