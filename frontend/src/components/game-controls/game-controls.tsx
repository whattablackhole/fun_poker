import { RefObject, useRef } from "react";
import { ActionType, ClientState, Player, PlayerPayload } from "../../types";
import ApiService from "../../services/api.service";
import "./game-controls.css";
import InputSlider from "./bet-slider";
import { Button, ButtonGroup, Grid } from "@mui/material";
import { Box } from "@react-three/drei";

function GameControls({ gameState, selfPlayer }: { gameState: ClientState, selfPlayer: Player }) {
    // const betClickHandler = (event: MouseEvent<HTMLButtonElement, globalThis.MouseEvent>, action: ActionType) => {
    //     event.preventDefault(); // Prevent default button behavior
    //     nextStepHandler(action);
    // };
    const betClickHandler = (event: any, action: ActionType) => {
        event.preventDefault(); // Prevent default button behavior
        nextStepHandler(action);
    };
    const betInputRef: RefObject<HTMLInputElement> = useRef(null);

    const nextStepHandler = (type: ActionType) => {
        let value = Number.parseInt(!!betInputRef.current?.value.length ? betInputRef.current?.value : "0");
        let payload = PlayerPayload.create({ action: { actionType: type, bet: value }, lobbyId: gameState?.lobbyId, playerId: gameState?.playerId });

        ApiService.sendMessage(payload);
    }

    return (
        <Box>
            <Grid container alignItems="flex-end" flexDirection="column" sx={{ gap: '10px' }}>
                <Grid  item sx={{ gap: '10px', display:'flex', flexDirection:'column' }}>
                    <div style={{display:'flex', justifyContent:'space-between'}}>
                        <Button
                            size="small"
                            sx={{
                                fontSize: '1rem',
                                background: 'linear-gradient(to bottom, lightgrey, darkgrey)',
                                boxShadow: '0 0 0 1px black, 0 0 0 2px grey',
                            }}
                            className="control-button"
                        // disabled={gameState?.currPlayerId !== selfPlayer.userId}
                        >
                            Min
                        </Button>
                        <Button
                            size="small"
                            sx={{
                                fontSize: '1rem',
                                background: 'linear-gradient(to bottom, lightgrey, darkgrey)',
                                boxShadow: '0 0 0 1px black, 0 0 0 2px grey',
                            }}
                            className="control-button"
                        // disabled={gameState?.currPlayerId !== selfPlayer.userId}
                        >
                            1/2
                        </Button>
                        <Button
                            size="small"
                            sx={{
                                fontSize: '1rem',
                                background: 'linear-gradient(to bottom, lightgrey, darkgrey)',
                                boxShadow: '0 0 0 1px black, 0 0 0 2px grey',
                            }}
                            className="control-button"
                        // disabled={gameState?.currPlayerId !== selfPlayer.userId}
                        >
                            Pot
                        </Button>
                        <Button
                            size="small"
                            sx={{
                                fontSize: '1rem',
                                background: 'linear-gradient(to bottom, lightgrey, darkgrey)',
                                boxShadow: '0 0 0 1px black, 0 0 0 2px grey',
                            }}
                            className="control-button"
                        // disabled={gameState?.currPlayerId !== selfPlayer.userId}
                        >
                            Max
                        </Button>
                    </div>

                    <InputSlider></InputSlider>
                </Grid>
                <Grid item sx={{ gap: '20px', display: 'flex' }}>
                    {/* <ButtonGroup variant="outlined" aria-label="betting options" sx={{
                        // backgroundColor: 'green',
                        // borderRadius: '8px',
                        // border: '2px solid black',
                        gap: '20px'
                    }}> */}
                    <Button
                        size="large"
                        sx={{
                            fontSize: '1.2rem',
                            background: 'linear-gradient(to bottom, lightgrey, darkgrey)',
                            boxShadow: '0 0 0 1px black, 0 0 0 4px grey, 0 0 0 5px black',
                            fontWeight: '800',
                            width: '150px',
                        }}
                        className="fold_button control-button"
                        // disabled={gameState?.currPlayerId !== selfPlayer.userId}
                        onClick={(e) => betClickHandler(e, ActionType.Fold)}
                    >
                        Fold
                    </Button>
                    <Button
                        size="large"
                        sx={{
                            fontSize: '1.2rem',
                            boxShadow: '0 0 0 1px black, 0 0 0 4px grey, 0 0 0 5px black',
                            fontWeight: '800',
                            background: 'linear-gradient(to bottom, lightgrey, darkgrey)',
                            width: '150px',

                        }}
                        className="call-check_button control-button"
                        // disabled={gameState?.currPlayerId !== selfPlayer.userId}
                        onClick={(e) => betClickHandler(e, ActionType.Call)}
                    >
                        Call
                    </Button>
                    <Button
                        size="large"
                        sx={{
                            fontSize: '1.2rem',
                            boxShadow: '0 0 0 1px black, 0 0 0 4px grey, 0 0 0 5px black',
                            fontWeight: '800',
                            background: 'linear-gradient(to bottom, lightgrey, darkgrey)',
                            width: '150px',
                        }}
                        className="raise_button control-button"
                        // disabled={gameState?.currPlayerId !== selfPlayer.userId}
                        onClick={(e) => betClickHandler(e, ActionType.Raise)}
                    >
                        Raise
                    </Button>
                    {/* </ButtonGroup> */}
                </Grid>
            </Grid>
        </Box>
    );

}

export default GameControls;

