import { Card, CardPair, ClientState, Street } from "../types";

const mockState = ClientState.create({
    cards: CardPair.create({ card1: Card.create({ value: 1, suit: 1 }), card2: Card.create({ value: 2, suit: 1 }) }),
    gameStatus: 1,
    latestWinners: [],
    lobbyId: 1,
    nextPlayerId: 0,
    playerId: 0,
    players: [
        {
            action: { actionType: 2, bet: 100 },
            bank: 10000,
            betInCurrentSeed: 100,
            country: "Belarus",
            userId: 0,
            userName: "Ruddy"
        },
        {
            action: { actionType: 0, bet: 0 },
            bank: 10000,
            betInCurrentSeed: 0,
            country: "Poland",
            userId: 1,
            userName: "Sindy"
        },
        {
            action: { actionType: 2, bet: 300 },
            bank: 10000,
            betInCurrentSeed: 300,
            country: "USA",
            userId: 2,
            userName: "Ronald"
        },
        {
            action: { actionType: 0, bet: 0 },
            bank: 10000,
            betInCurrentSeed: 0,
            country: "England",
            userId: 3,
            userName: "Josh"
        },
        {
            action: { actionType: 0, bet: 0 },
            bank: 10000,
            betInCurrentSeed: 0,
            country: "China",
            userId: 4,
            userName: "Si Lue"
        },
        {
            action: { actionType: 0, bet: 0 },
            bank: 10000,
            betInCurrentSeed: 0,
            country: "Japan",
            userId: 5,
            userName: "Woghn Gee"
        },
        {
            action: { actionType: 0, bet: 0 },
            bank: 10000,
            betInCurrentSeed: 0,
            country: "Belarus",
            userId: 6,
            userName: "Alexander"
        },
        {
            action: { actionType: 0, bet: 0 },
            bank: 10000,
            betInCurrentSeed: 0,
            country: "Germany",
            userId: 7,
            userName: "Carl Fritz"
        },
        {
            action: { actionType: 0, bet: 0 },
            bank: 10000,
            betInCurrentSeed: 0,
            country: "New Zeland",
            userId: 8,
            userName: "Joe Hister"
        }
    ],
    street: Street.create({ cards: [{ value: 3, suit: 2 }, { value: 8, suit: 2 }, { value: 6, suit: 1 }] }),

})

export default mockState;