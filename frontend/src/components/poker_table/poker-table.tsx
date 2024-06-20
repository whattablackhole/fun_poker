// @ts-nocheck
import { MouseEvent, RefObject, useEffect, useRef, useState, } from 'react';
import PokerCard from '../poker_card/poker-card.tsx';
import { ClientState } from '../../types/client_state.ts';
import { Card } from '../../types/card.ts';
import ApiService from '../../services/api.service.ts';
import { ActionType, Player, PlayerPayload } from '../../types/player.ts';
import TimerBanner from '../timer_banner/timer-banner.tsx';
import { StreetStatus } from '../../types/game_state.ts';
import mockState from '../../mocks/client-state.mock.ts';

type PlayerId = number;
type BetAmount = number;

type BetHistoryMap = Map<PlayerId, { [key in StreetStatus]: BetAmount }>;

interface BetHistory {
    bank_on_prev_street: number,
    betHistoryMap: BetHistoryMap,
    prevStreet?: StreetStatus
}

function PokerTable(init_state: ClientState) {
    const canvasRef: RefObject<HTMLCanvasElement> = useRef(null);
    const betInputRef: RefObject<HTMLInputElement> = useRef(null);
    init_state = mockState;
    console.log(mockState);
    const [state, setState] = useState(init_state);
    // TODO: make readonly
    const stateRef = useRef(state);

    const [boardCards, setBoardCards] = useState(init_state.street?.cards);
    const [players, setPlayers] = useState(center_players_by_self(init_state));
    const [betHistory, setBetHistory] = useState<BetHistory>(calculateBetHistory(init_state, { bank_on_prev_street: 0, betHistoryMap: new Map() }, false));

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

    const betClickHandler = (event: MouseEvent<HTMLButtonElement, globalThis.MouseEvent>, action: ActionType) => {
        event.preventDefault(); // Prevent default button behavior
        nextStepHandler(action);
    };
    const nextStepHandler = (type: ActionType) => {
        let value = Number.parseInt(!!betInputRef.current?.value.length ? betInputRef.current?.value : "0");
        let payload = PlayerPayload.create({ action: { actionType: type, bet: value }, lobbyId: state.lobbyId, playerId: state.playerId });

        ApiService.sendMessage(payload);
    }
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

        let subscription = ApiService.clientStateObserver.subscribe(async (newState: ClientState) => {
            console.log(newState);

            await processNewState(newState, stateRef.current, setBoardCards, setBetHistory, setPlayers, betHistory);
            setState(newState);
            stateRef.current = newState;

        })

        return () => {
            ApiService.clientStateObserver.unsubscribe(subscription);
        }
    }, []);

    if (!state) {
        return <div>Loading...</div>
    }

    // TODO: move to effect the code below
    if (canvas) {
        canvas.clearRect(0, 0, canvas_width, canvas_height)
        canvas.save();
        canvas.scale(scaleX, scaleY);
        canvas.beginPath();

        canvas.arc(descaledX, descaledY, radius, 0, Math.PI * 2);
        canvas.fillStyle = 'blue';
        canvas.fill();
        canvas.strokeStyle = 'green';
        canvas.lineWidth = 10;
        canvas.stroke();
        canvas.restore();
    }

    const playerPositions = calculatePlayerPositionsFromCanvas(radius, centerX, centerY, scaleX, scaleY);

    let cards = [
        state.cards!.card1!,
        state.cards!.card2!
    ]

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
        betHistory.betHistoryMap.forEach((history, playerId) => {
            let amount = history[state.street!.streetStatus];
            if (amount > 0) {
                let index = get_index_by_player_id(players, playerId);
                const angle = ((2 * Math.PI) / players_amount) * index + Math.PI / 2;

                drawChips(centerX, centerY, amount, angle, canvas);
            }

        })
    }

    if (betHistory.bank_on_prev_street > 0 && canvas) {
        const angle = Math.PI / 2;
        drawChips(centerX, centerY, betHistory.bank_on_prev_street, angle, canvas, 50)
    }


    return (
        <div style={{ backgroundImage: "url('./src/assets/background.png')", backgroundSize: "cover", width: "100%", height: "100%", display: "flex", justifyContent: "center" }}>
            <div style={{ position: 'relative' }}>
                {playerPositions.map((position, index) => (
                    <div key={index} style={{
                        position: 'absolute', top: position.y, left: position.x, display: 'flex', flexDirection: 'column',
                    }}>
                        <div style={{ display: 'flex', flexDirection: 'column', width: "fit-content" }}>
                            <div style={{ display: 'flex' }}>
                                {cards.map(({ suit, value }) => {
                                    return <PokerCard cardSuit={index === 0 ? suit : undefined} cardValue={index === 0 ? value : undefined} />
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
                    {boardCards?.map(({ suit, value }) => {
                        return <PokerCard cardSuit={suit} cardValue={value} />
                    })}
                </div>


                <canvas ref={canvasRef} />

            </div>
        </div>

    );
}


async function processNewState(newState: ClientState, prevState: ClientState, setBoardCards: React.Dispatch<React.SetStateAction<Card[] | undefined>>, setBetHistory: React.Dispatch<React.SetStateAction<BetHistory>>, setPlayers: React.Dispatch<React.SetStateAction<Player[]>>, betHistory: BetHistory) {

    // TODO: figure out how to set actual player banks on processFlopAutomatically
    // maybe worth to do it on backend side in future
    if (newState.showdownOutcome && newState.showdownOutcome.processFlopAutomatically) {
        // newState contains updated banks after the cycle
        // reimplement logic to escape hacks
        let lastAction = newState.actionHistory[newState.actionHistory.length - 1];
        let player = prevState.players.find((p) => p.userId === lastAction.playerId);
        if (player) {
            player.action = lastAction;
            if (lastAction.actionType == ActionType.Call || lastAction.actionType == ActionType.Raise) {
                if (player) {
                    player.bank -= lastAction.bet;
                }
            }
        }

        setPlayers(center_players_by_self(prevState));
        const processedHistory = calculateBetHistory(newState, betHistory, true);
        setBetHistory(processedHistory);

        await animateShowdown(newState, prevState, setBoardCards);
    } else {
        const processedHistory = calculateBetHistory(newState, betHistory, false);

        setBetHistory(processedHistory);
        setBoardCards(newState.street?.cards);
    }
    setPlayers(center_players_by_self(newState));

    return newState;
}

async function animateShowdown(newState: ClientState, prevState: ClientState, setBoardCards: React.Dispatch<React.SetStateAction<Card[] | undefined>>) {
    const startIndex = prevState.street?.cards.length ?? 0;
    if (newState.showdownOutcome) {
        let cardsToShow = newState.showdownOutcome.streetHistory!.cards.slice(startIndex);
        let currBoardCards = prevState.street?.cards ?? [];
        for (let c of cardsToShow) {
            currBoardCards.push(c);
            await setBoardCardsWithDelay(1000, setBoardCards, currBoardCards);

        }
        await new Promise(resolve => setTimeout(resolve, 3000))
    }
}

async function setBoardCardsWithDelay(delay: number, setBoardCards: React.Dispatch<React.SetStateAction<Card[] | undefined>>, cards: Card[]) {
    return await new Promise((resolve) => {
        setTimeout(() => {
            setBoardCards([...cards]);
            resolve(undefined);
        }, delay)
    })
}

function calculateBetHistory(newState: ClientState, betHistory: BetHistory, automatedShowdown: boolean): BetHistory {
    betHistory.betHistoryMap.clear();
    betHistory.bank_on_prev_street = 0;
    newState.actionHistory.forEach((action) => {
        let history = betHistory.betHistoryMap.get(action.playerId) || { 0: 0, 1: 0, 2: 0, 3: 0 }
        history[action.streetStatus] += action.bet;
        betHistory.betHistoryMap.set(action.playerId, history);
    })
    if (newState.street?.streetStatus !== 0) {
        betHistory.bank_on_prev_street = calculateTotalBankOfPrevStreets(betHistory.betHistoryMap, newState.street!.streetStatus - 1)
    }
    // NOTE: maybe not needed, rethink
    if (automatedShowdown) {
        betHistory.betHistoryMap.clear();
    }
    return betHistory;
}

function calculateTotalBankOfPrevStreets(betHistory: BetHistoryMap, prevStreet: StreetStatus): number {
    let totalBank = 0;

    while (prevStreet > -1) {
        for (const [, object] of betHistory) {
            totalBank += object[prevStreet];
        }
        --prevStreet;
    }

    return totalBank;
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