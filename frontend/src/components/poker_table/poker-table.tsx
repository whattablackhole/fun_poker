import { MouseEvent, RefObject, useEffect, useRef, useState } from 'react';
import PokerCard from '../poker_card/poker-card.tsx';
import { ClientState } from '../../types/client-state.ts';
import { CardValue } from '../../types/card.ts';
import ApiService from '../../services/api.service.ts';
import { ActionType, PlayerPayload } from '../../types/player.ts';

function PokerTable(init_state: ClientState) {
    const canvasRef: RefObject<HTMLCanvasElement> = useRef(null);
    const betInputRef: RefObject<HTMLInputElement> = useRef(null);
    const [state, setState] = useState(init_state);
    let canvas_width = 1000;
    let canvas_height = 700;
    const centerX = canvas_width / 2;
    const centerY = canvas_height / 2;
    const radius = 150;
    const scaleX = 1.7;
    const scaleY = 1.2;
    let scaledX = centerX / scaleX;
    let scaledY = centerY / scaleY;
    useEffect(() => {
        const canvas: HTMLCanvasElement = canvasRef.current as HTMLCanvasElement;
        // read about init_state param and useState default arg, is it in sync?
        let subscription = ApiService.clientStateObserver.subscribe((newState: ClientState) => {
            console.log(newState);
            if (newState.latestWinners.some((w) => w.userId === newState.playerId)) {
                console.log("You won this!");
            }
            setState(newState);
        })
        if (canvas) {
            const ctx = canvas.getContext('2d');

            if (ctx) {
                canvas.width = canvas_width;
                canvas.height = canvas_height;

                ctx.scale(scaleX, scaleY);
                ctx.beginPath();
                ctx.arc(scaledX, scaledY, radius, 0, Math.PI * 2);
                ctx.strokeStyle = 'green';
                ctx.lineWidth = 10;
                ctx.stroke();

                // for test
                const players = 9;
                const player_index = 3;
                const bet_points = 400;
                const angle = ((2 * Math.PI) * player_index) / players + Math.PI / 2;
                drawChips(scaledX, scaledY, bet_points, angle, ctx);
            }
        } else {
            console.error("canvas is null", canvas)
        }
        return () => {
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
    const betClickHandler = (event: MouseEvent<HTMLButtonElement, globalThis.MouseEvent>, action: ActionType) => {
        event.preventDefault(); // Prevent default button behavior
        nextStepHandler(action);
    };
    const nextStepHandler = (type: ActionType) => {
        let value = Number.parseInt(!!betInputRef.current?.value.length ? betInputRef.current?.value : "0");
        let payload = PlayerPayload.create({ action: { actionType: type, bet: value }, lobbyId: state.lobbyId, playerId: state.playerId });

        ApiService.sendMessage(payload);

    }
    return (
        <div style={{ position: 'relative', marginLeft: '200px' }}>
            {playerPositions.map((position, index) => (
                <div key={index} style={{
                    position: 'absolute', top: position.y, left: position.x, display: 'flex', flexDirection: 'column',
                }}>
                    <div style={{ display: 'flex', flexDirection: 'column', width: "fit-content" }}>
                        <div style={{ display: 'flex' }}>
                            {cards.map(({ suit, value }) => {
                                return <PokerCard cardSuit={index === 0 ? suit : null} cardValue={index === 0 ? value : null} />
                            })}
                        </div>

                        <div style={{ alignSelf: 'center', width: "100%", textAlign: "center", backgroundColor: 'wheat', marginTop: '-20px', }}>
                            <div>
                                {(state?.players[index]?.bank ?? "100 000") + " chips"}
                            </div>
                            <div>
                                {state?.players[index]?.userName ?? "NickName"}
                            </div>
                        </div>
                    </div>

                    {index === 0 ? (
                        <form style={{ display: "flex", alignItems: "flex-end" }}>
                            <input ref={betInputRef} type="number" min="0" max={state?.players.find((p) => p.userId == state.playerId)?.bank || 0}></input>
                            <button
                                disabled={state?.nextPlayerId !== state?.playerId}
                                style={{ width: "100px", height: "50px", alignSelf: "flex-end" }}
                                onClick={(e) => betClickHandler(e, ActionType.Fold)}
                            >
                                Fold
                            </button>
                            <button
                                disabled={state?.nextPlayerId !== state?.playerId}
                                style={{ width: "100px", height: "50px", alignSelf: "flex-end" }}
                                onClick={(e) => betClickHandler(e, ActionType.Call)}
                            >
                                Call
                            </button>
                            <button
                                disabled={state?.nextPlayerId !== state?.playerId}
                                style={{ width: "100px", height: "50px", alignSelf: "flex-end" }}
                                onClick={(e) => betClickHandler(e, ActionType.Raise)}
                            >
                                Raise
                            </button>
                        </form>

                    ) : null}
                </div>
            ))}
            <div style={{ position: 'absolute', top: scaledY, left: scaledX, display: 'flex' }}>
                {state?.street?.cards.map(({ suit, value }) => {
                    return <PokerCard cardSuit={suit} cardValue={value} />
                })}
            </div>


            <canvas ref={canvasRef} />

        </div>
    );
}
function drawChips(centerX: number, centerY: number, points: number, angle: number, ctx: CanvasRenderingContext2D) {
    const x = centerX + Math.cos(angle) * 120;
    const y = centerY + Math.sin(angle) * 120;
    // TODO: parse points into digit groups and draw them accordingly 
    drawChip(x, y, ctx);
    drawChip(x, y - 3, ctx);
    drawChip(x, y - 6, ctx);
    drawChip(x, y - 9, ctx);
    if (points) {
        ctx.fillStyle = 'black';
        ctx.font = '12px Arial';
        ctx.fillText(`${points} chips`, x, y + 15);
    }
}
function drawChip(x: number, y: number, ctx: CanvasRenderingContext2D, rad: number = 7) {
    ctx.beginPath();
    ctx.arc(x, y, rad, 0, Math.PI * 2);
    ctx.strokeStyle = 'blue';
    ctx.lineWidth = 1;
    ctx.fillStyle = 'green';
    ctx.fill('evenodd');
    ctx.stroke();
    ctx.closePath();
    let numTriangles = 6;
    let angleStep = (Math.PI * 2) / numTriangles;

    for (let i = 0; i < numTriangles; i++) {
        ctx.beginPath();
        let x1 = x + Math.cos(angleStep * i) * rad;
        let y1 = y + Math.sin(angleStep * i) * rad;
        ctx.moveTo(x1, y1);
        let endX = x + Math.cos(angleStep * i) * (rad - 2);
        let endY = y + Math.sin(angleStep * i) * (rad - 2);
        ctx.lineTo(endX, endY);
        ctx.strokeStyle = 'red';
        ctx.lineWidth = 2;
        ctx.stroke();
        ctx.fillStyle = 'black';
        ctx.font = '10px Arial';
        ctx.textAlign = 'center';
        ctx.textBaseline = 'middle';
        ctx.fillText("5", x, y);
        ctx.closePath();
    }
}
function calculatePlayerPositionsFromCanvas(radius: number, xCord: number, yCord: number, scaleX: number, scaleY: number) {
    const numPlayers = 9;
    const positions = [];
    for (let i = 0; i < numPlayers; i++) {
        const angle = ((2 * Math.PI) * i) / numPlayers + Math.PI / 2;
        const cos_ratio = Math.cos(angle);
        const sin_ratio = Math.sin(angle);
        let y_fix = sin_ratio < 0 ? -150 : 0;
        let x_fix = cos_ratio < 0 ? -180 : 0;
        // y_fix x_fix depend on card width and height sizes;
        if (i === 0) {
            x_fix = -75;
        }
        const x = xCord + x_fix + radius * cos_ratio * scaleX;
        const y = yCord + y_fix + radius * sin_ratio * scaleY;
        positions.push({ x, y });
    }
    return positions;
}

export default PokerTable;