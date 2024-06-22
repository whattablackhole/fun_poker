import GameStateProcessHelper from "../helpers/game-state-helper";
import { Card, ClientState, Player } from "../types";
import BetHistory from "../types/bet-history";

class GameStateService {
    static async processNewState(newState: ClientState, prevState: ClientState | undefined, betHistory: BetHistory, setBoardCards: React.Dispatch<React.SetStateAction<Card[] | undefined>>, setBetHistory: React.Dispatch<React.SetStateAction<BetHistory>>, setPlayers: React.Dispatch<React.SetStateAction<Player[] | undefined>>) {
        if (prevState == null) {
            this.setupPlayers(newState, setPlayers);
            betHistory.calculateBetHistory(newState, false);
            setBetHistory(betHistory);
            setBoardCards(newState.street?.cards);

            return newState;
        }

        // TODO: figure out how to set actual player banks on processFlopAutomatically
        // maybe worth to do it on backend side in future

        betHistory.calculateBetHistory(newState, !!newState.showdownOutcome);
        setBetHistory(betHistory);

        if (newState.showdownOutcome && newState.showdownOutcome.processFlopAutomatically) {
            await this.animateShowdown(newState, prevState, setBoardCards);
        } else {
            setBoardCards(newState.street?.cards);
        }
        this.setupPlayers(newState, setPlayers);

        return newState;
    }

    private static setupPlayers(state: ClientState, setPlayers: React.Dispatch<React.SetStateAction<Player[] | undefined>>,) {
        let selfId = state.playerId;

        let player = state.players.find((p)=>p.userId == selfId);

        if (player && state.cards) {
            player.cards = state.cards;
        }

        setPlayers(GameStateProcessHelper.center_players_by_self(state.players, state.playerId));
    }

    static async animateShowdown(newState: ClientState, prevState: ClientState, setBoardCards: React.Dispatch<React.SetStateAction<Card[] | undefined>>) {
        const startIndex = prevState.street?.cards.length ?? 0;
        if (newState.showdownOutcome) {
            let cardsToShow = newState.showdownOutcome.streetHistory!.cards.slice(startIndex);
            let currBoardCards = prevState.street?.cards ?? [];
            for (let c of cardsToShow) {
                currBoardCards.push(c);
                await this.setBoardCardsWithDelay(1000, setBoardCards, currBoardCards);

            }
            await new Promise(resolve => setTimeout(resolve, 3000))
        }
    }


    static async setBoardCardsWithDelay(delay: number, setBoardCards: React.Dispatch<React.SetStateAction<Card[] | undefined>>, cards: Card[]) {
        return await new Promise((resolve) => {
            setTimeout(() => {
                setBoardCards([...cards]);
                resolve(undefined);
            }, delay)
        })
    }

}

export default GameStateService

