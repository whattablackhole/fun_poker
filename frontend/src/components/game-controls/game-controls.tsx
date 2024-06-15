import { useState } from "react";
import { ActionType, ClientState, Player } from "../../types";
import "./game-controls.css";
import InputSlider from "./bet-slider";
import { Button, Grid } from "@mui/material";
import { Box } from "@react-three/drei";

function GameControls({ gameState, betClickHandler }: { gameState: ClientState, betClickHandler: (value: number, type: ActionType) => void }) {
    const [betSizeInputValue, setBetSizeInputValue] = useState<number>(0);

    

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

                    <InputSlider onValueChange={setBetSizeInputValue}></InputSlider>
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
                        onClick={(e) => betClickHandler(0, ActionType.Fold)}
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
                        onClick={(e) => betClickHandler(betSizeInputValue, ActionType.Call)}
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
                        onClick={(e) => betClickHandler(betSizeInputValue, ActionType.Raise)}
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

