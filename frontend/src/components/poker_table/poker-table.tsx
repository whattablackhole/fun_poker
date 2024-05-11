import { MouseEvent, RefObject, useEffect, useRef, useState, } from 'react';
import PokerCard from '../poker_card/poker-card.tsx';
import { ClientState, StreetStatus } from '../../types/client-state.ts';
import { CardValue } from '../../types/card.ts';
import ApiService from '../../services/api.service.ts';
import { ActionType, Player, PlayerPayload } from '../../types/player.ts';
import TimerBanner from '../timer_banner/timer-banner.tsx';

type PlayerId = number;
type BetAmount = number;
type BetHistoryMap = Map<PlayerId, BetAmount>;

interface BetHistory {
    bank_on_curr_street: number,
    betHistoryMap: BetHistoryMap
}

function PokerTable(init_state: ClientState) {
    const canvasRef: RefObject<HTMLCanvasElement> = useRef(null);
    const betInputRef: RefObject<HTMLInputElement> = useRef(null);
    const [state, setState] = useState(init_state);
    const [betHistory, setBetHistory] = useState<BetHistory>(() => {
        const initialBetHistory: BetHistoryMap = new Map<number, number>();
        const bigIndex = get_index_by_player_id(init_state.players, init_state.currBigBlindId);
        const smallIndex = get_index_by_player_id(init_state.players, init_state.currSmallBlindId);

        if (smallIndex !== -1) {
            initialBetHistory.set(init_state.currSmallBlindId, init_state.players[smallIndex].action!.bet);
        }
        if (bigIndex !== -1) {
            initialBetHistory.set(init_state.currBigBlindId, init_state.players[bigIndex].action!.bet);
        }

        return { bank_on_curr_street: 0, betHistoryMap: initialBetHistory };
    });

    const [canvas, setCanvas] = useState<CanvasRenderingContext2D | null>(null);
    let canvas_width = 1000;
    let canvas_height = 800;
    const centerX = canvas_width / 2;
    const centerY = canvas_height / 2;
    const radius = 200;
    const scaleX = 1.7;
    const scaleY = 1.2;
    let descaledX = centerX / scaleX;
    let descaledY = centerY / scaleY;

    useEffect(() => {
        const canvas: HTMLCanvasElement = canvasRef.current as HTMLCanvasElement;

        if (canvas) {
            const ctx = canvas.getContext('2d');
            canvas.width = canvas_width;
            canvas.height = canvas_height;
            setCanvas(ctx);

        } else {
            console.error("canvas is null", canvas)
        }

        let subscription = ApiService.clientStateObserver.subscribe((newState: ClientState) => {
            console.log(newState);
            if (newState.latestWinners.some((w) => w.userId === newState.playerId)) {
                console.log("You won this!");
            }

            setState(prevState => {
                const processedHistory = processBetHistoryState(newState, prevState, betHistory);

                setBetHistory(processedHistory);

                return newState;
            });
        })

        return () => {
            ApiService.clientStateObserver.unsubscribe(subscription);
        }
    }, []);

    if (!state) {
        return <div>Loading...</div>
    }


    if (canvas) {
        canvas.clearRect(0, 0, canvas_width, canvas_height)
        canvas.save();
        canvas.scale(scaleX, scaleY);
        canvas.beginPath();
        canvas.arc(descaledX, descaledY, radius, 0, Math.PI * 2);
        canvas.strokeStyle = 'green';
        canvas.lineWidth = 10;
        canvas.stroke();
        canvas.restore();
    }

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
    const players = center_players_by_self(state);
    const players_amount = 9;
    if (canvas) {
        let button_player_index = get_index_by_player_id(players, state.currButtonId);
        let step = Math.PI * 2 / players_amount;
        let angle = step * button_player_index + Math.PI / 2;
        let x = centerX + Math.cos(angle) * (radius - 40) * scaleX;
        let y = centerY + Math.sin(angle) * (radius - 40) * scaleY;
        drawButton(x, y, canvas, 14);
    }

    const selfPlayer = players[0];




    if (canvas) {
        betHistory.betHistoryMap.forEach((bet, playerId) => {
            let index = get_index_by_player_id(players, playerId);
            const angle = ((2 * Math.PI) / players_amount) * index + Math.PI / 2;

            drawChips(centerX, centerY, bet, angle, canvas);
        })
    }

    if (betHistory.bank_on_curr_street > 0 && canvas) {
        const angle = Math.PI / 2;
        drawChips(centerX, centerY, betHistory.bank_on_curr_street, angle, canvas, 50)
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
                                {(players[index]?.bank ?? "100 000") + " chips"}
                            </div>
                            <div>
                                {players[index]?.userName ?? "NickName"}
                            </div>
                            {state && players[index]?.userId === state.currPlayerId ?
                                <div>
                                    <TimerBanner timeLeft={100} />
                                </div>

                                : null}
                        </div>
                    </div>

                    {index === 0 ? (
                        <form style={{ display: "flex", alignItems: "flex-end" }}>
                            <input ref={betInputRef} type="number" min="0" max={selfPlayer.bank || 0}></input>
                            <button
                                disabled={state?.currPlayerId !== selfPlayer.userId}
                                style={{ width: "100px", height: "50px", alignSelf: "flex-end" }}
                                onClick={(e) => betClickHandler(e, ActionType.Fold)}
                            >
                                Fold
                            </button>
                            <button
                                disabled={state?.currPlayerId !== selfPlayer.userId}
                                style={{ width: "100px", height: "50px", alignSelf: "flex-end" }}
                                onClick={(e) => betClickHandler(e, ActionType.Call)}
                            >
                                Call
                            </button>
                            <button
                                disabled={state?.currPlayerId !== selfPlayer.userId}
                                style={{ width: "100px", height: "50px", alignSelf: "flex-end" }}
                                onClick={(e) => betClickHandler(e, ActionType.Raise)}
                            >
                                Raise
                            </button>
                        </form>

                    ) : null}
                </div>
            ))}
            <div style={{ position: 'absolute', top: descaledY - 50, left: descaledX, display: 'flex' }}>
                {state?.street?.cards.map(({ suit, value }) => {
                    return <PokerCard cardSuit={suit} cardValue={value} />
                })}
            </div>


            <canvas ref={canvasRef} />

        </div>
    );
}




function calculateBankSpot(betHistory: Map<number, number>): number {
    let spot = Array.from(betHistory.values()).reduce((p, k) => p + k, 0);
    return spot;
}

function calculateLastPlayerAction(newState: ClientState, oldState: ClientState, betHistory: BetHistory) {
    let index = get_index_by_player_id(newState.players, oldState.currPlayerId);
    let player = newState.players[index];
    if (player.action?.actionType == ActionType.Call || player.action?.actionType == ActionType.Raise) {

        if (betHistory.betHistoryMap.has(player.userId)) {
            let el = betHistory.betHistoryMap.get(player.userId);
            betHistory.betHistoryMap.set(player.userId, el! + (player.action?.bet ?? 0));
        } else {
            betHistory.betHistoryMap.set(player.userId, player.action?.bet ?? 0);
        }
    }
}


function processBetHistoryState(newState: ClientState, oldState: ClientState, betHistory: BetHistory): BetHistory {
    if (newState.street?.streetStatus !== oldState.street?.streetStatus) {
        if (newState.street?.streetStatus === StreetStatus.Preflop) {
            betHistory.betHistoryMap.clear();

            const bigIndex = get_index_by_player_id(newState.players, newState.currBigBlindId);
            const smallIndex = get_index_by_player_id(newState.players, newState.currSmallBlindId);

            if (smallIndex !== -1) {
                betHistory.betHistoryMap.set(newState.currSmallBlindId, newState.players[smallIndex].action!.bet);
            }
            if (bigIndex !== -1) {
                betHistory.betHistoryMap.set(newState.currBigBlindId, newState.players[bigIndex].action!.bet);
            }
            betHistory.bank_on_curr_street = 0;
            return betHistory;
        }
        calculateLastPlayerAction(newState, oldState, betHistory);
        let currBank = calculateBankSpot(betHistory.betHistoryMap);
        betHistory.betHistoryMap.clear();
        betHistory.bank_on_curr_street = currBank;
        return betHistory;
    }

    calculateLastPlayerAction(newState, oldState, betHistory);
    return betHistory;
}


function center_players_by_self(state: ClientState): Player[] {
    let index = get_index_by_player_id(state.players, state.playerId);

    if (index !== -1) {
        return [...state.players.slice(index), ...state.players.slice(0, index)];

    } else {
        return [];
    }
}

function get_index_by_player_id(players: Player[], id: number): number {
    return players.findIndex((p) => p.userId == id)
}

function drawButton(x: number, y: number, ctx: CanvasRenderingContext2D, rad: number = 10) {
    const shadowColor = '#D5B60A';
    const buttonColor = '#FFCC00 ';

    ctx.beginPath();
    ctx.save();
    ctx.scale(1.1, 0.8)


    let descaledX = x / 1.1
    let descaledY = y / 0.8;
    ctx.arc(descaledX, descaledY + 5, rad, 0, Math.PI * 2);
    ctx.fillStyle = shadowColor;
    ctx.fill();
    ctx.closePath();

    ctx.beginPath();

    ctx.arc(x / 1.1, y / 0.8, rad, 0, Math.PI * 2);
    ctx.strokeStyle = buttonColor;
    ctx.lineWidth = 1;
    ctx.fillStyle = buttonColor;
    ctx.fill('evenodd');
    ctx.stroke();
    const gradient = ctx.createRadialGradient(descaledX, descaledY, 0, descaledX, descaledY, rad);
    gradient.addColorStop(0, buttonColor);
    gradient.addColorStop(1, '#FFE066');
    ctx.fillStyle = gradient;
    ctx.fill();
    ctx.fillStyle = 'black';
    ctx.font = 'bold 15pt Courier';
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    ctx.fillText(`D`, x / 1.1, y / 0.8,);


    ctx.closePath();
    ctx.restore();
}

function drawChips(centerX: number, centerY: number, points: number, angle: number, ctx: CanvasRenderingContext2D, radius = 120) {
    const x = centerX + Math.cos(angle) * radius * 1.7;
    const y = centerY + Math.sin(angle) * radius * 1.2;
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