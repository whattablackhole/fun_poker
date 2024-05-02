import { RefObject, useEffect, useRef, useState } from 'react';
import PokerCard from '../poker_card/poker-card.tsx';
import { ClientState } from '../../types/client-state.ts';
import { CardValue } from '../../types/card.ts';
import ApiService from '../../services/api.service.ts';
import { ActionType, PlayerPayload } from '../../types/player.ts';

function PokerTable(init_state: ClientState) {
    const canvasRef: RefObject<HTMLCanvasElement> = useRef(null);
    const [state, setState] = useState(init_state);
    let canvas_width = 1000;
    let canvas_height = 1000;
    const centerX = canvas_width / 2;
    const centerY = canvas_height / 2;
    const radius = 250;
    const scaleX = 1.7;
    const scaleY = 1.2;

    useEffect(() => {
        const canvas: HTMLCanvasElement = canvasRef.current as HTMLCanvasElement;
        // read about init_state param and useState default arg, is it in sync?
        let subscription = ApiService.clientStateObserver.subscribe((newState: ClientState) => {
            setState(newState);
        })
        if (canvas) {
            const ctx = canvas.getContext('2d');

            if (ctx) {
                canvas.width = canvas_width;
                canvas.height = canvas_height;


                ctx.scale(scaleX, scaleY);
                ctx.beginPath();
                ctx.arc(centerX / scaleX, centerY / scaleY, radius, 0, Math.PI * 2);
                ctx.strokeStyle = 'green';
                ctx.lineWidth = 10;
                ctx.stroke();
            }
        } else {
            console.error("canvas is null", canvas)
        }
        return ()=> {
            ApiService.clientStateObserver.unsubscribe(subscription);
        }
    }, []);

    const playerPositions = calculatePlayerPositionsFromCanvas(radius, centerX, centerY, scaleX, scaleY);
    // for debug purposes i mocking it if state empty;
    let cards = state ? [
        state.cards!.card1!,
        state.cards!.card2!
    ] : [
        { suit: 0, value: CardValue.Ace },
        { suit: 0, value: CardValue.Queen }
    ];
    const nextStepHandler = () => {
        let payload = PlayerPayload.create({ action: { actionType: ActionType.Raise }, lobbyId: state.lobbyId, playerId: state.playerId });
        ApiService.sendMessage(payload);
    }
    return (
        <div style={{ position: 'relative' }}>
            {playerPositions.map((position, index) => (
                <div key={index} style={{ position: 'absolute', top: position.y, left: position.x, display: 'flex' }}>
                    {cards.map(({ suit, value }) => {
                        return <PokerCard cardSuit={index === 0 ? suit : null} cardValue={index === 0 ? value : null} />
                    })}
                    {index === 0 ? (
                        <button disabled={state?.nextPlayerId !== state?.playerId} style={{ width: "100px", height: "50px", alignSelf: 'flex-end' }} onClick={nextStepHandler}>NEXT STEP</button>
                    ) : null}
                </div>
            ))}
            <div style={{ position: 'absolute', top: centerY / scaleY, left: centerX / scaleX, display: 'flex' }}>
                {state.street?.cards.map(({ suit, value }) => {
                    return <PokerCard cardSuit={suit} cardValue={value} />
                })}
            </div>


            <canvas ref={canvasRef} />

        </div>
    );
}

function calculatePlayerPositionsFromCanvas(radius: number, xCord: number, yCord: number, scaleX: number, scaleY: number) {
    const numPlayers = 9;
    const positions = [];
    for (let i = 0; i < numPlayers; i++) {
        const angle = ((2 * Math.PI) * i) / numPlayers + Math.PI / 2;
        const cos_ratio = Math.cos(angle);
        const sin_ratio = Math.sin(angle);
        const y_fix = sin_ratio < 0 ? -150 : 0;

        const x = xCord + radius * cos_ratio * scaleX;
        const y = yCord + y_fix + radius * sin_ratio * scaleY;
        positions.push({ x, y });
    }
    return positions;
}

export default PokerTable;