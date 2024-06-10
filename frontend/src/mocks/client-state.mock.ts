import { Card, CardPair, ClientState } from "../types";
import { Street } from "../types/game_state";

const mockState = ClientState.create({
    cards: CardPair.create({ card1: Card.create({ value: 1, suit: 1 }), card2: Card.create({ value: 2, suit: 1 }) }),
    gameStatus: 1,
    canRaise: false,
    currButtonId: 1,
    currPlayerId: 1,
    currSmallBlindId: 2,
    minAmountToRaise: 100,
    amountToCall: 100,
    currBigBlindId: 3,
    lobbyId: 1,
    playerId: 0,
    players: [
        {
            action: { actionType: 2, bet: 100 },
            bank: 10000,
            betInCurrentSeed: 100,
            country: "BY",
            userId: 0,
            userName: "Ruddy"
        },
        {
            action: { actionType: 0, bet: 0 },
            bank: 10000,
            betInCurrentSeed: 0,
            country: "PL",
            userId: 1,
            userName: "Sindy"
        },
        {
            action: { actionType: 2, bet: 300 },
            bank: 10000,
            betInCurrentSeed: 300,
            country: "US",
            userId: 2,
            userName: "Ronald"
        },
        {
            action: { actionType: 0, bet: 0 },
            bank: 10000,
            betInCurrentSeed: 0,
            country: "GB",
            userId: 3,
            userName: "Josh"
        },
        {
            action: { actionType: 0, bet: 0 },
            bank: 10000,
            betInCurrentSeed: 0,
            country: "CN",
            userId: 4,
            userName: "Si Lue"
        },
        {
            action: { actionType: 0, bet: 0 },
            bank: 10000,
            betInCurrentSeed: 0,
            country: "JP",
            userId: 5,
            userName: "Woghn Gee"
        },
        {
            action: { actionType: 0, bet: 0 },
            bank: 10000,
            betInCurrentSeed: 0,
            country: "BY",
            userId: 6,
            userName: "Alexander"
        },
        {
            action: { actionType: 0, bet: 0 },
            bank: 10000,
            betInCurrentSeed: 0,
            country: "DE",
            userId: 7,
            userName: "Carl Fritz"
        },
        {
            action: { actionType: 0, bet: 0 },
            bank: 10000,
            betInCurrentSeed: 0,
            country: "NZ",
            userId: 8,
            userName: "Joe Hister"
        }
    ],
    street: Street.create({ cards: [{ value: 3, suit: 2 }, { value: 8, suit: 2 }, { value: 6, suit: 1 }] }),

})

export default mockState;