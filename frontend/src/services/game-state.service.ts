import GameStateProcessHelper from "../helpers/game-state-helper";
import { ActionType, Card, ClientState, Player } from "../types";
import BetHistory from "../types/bet-history";

class GameStateService {
    static async processNewState(newState: ClientState, prevState: ClientState | undefined, betHistory: BetHistory, setBoardCards: React.Dispatch<React.SetStateAction<Card[] | undefined>>, setBetHistory: React.Dispatch<React.SetStateAction<BetHistory>>, setPlayers: React.Dispatch<React.SetStateAction<Player[] | undefined>>) {
        if (prevState == null) {
            setPlayers(GameStateProcessHelper.center_players_by_self(newState.players, newState.playerId));
            betHistory.calculateBetHistory(newState, false);
            setBetHistory(betHistory);
            setBoardCards(newState.street?.cards);

            return newState;
        }

        // TODO: figure out how to set actual player banks on processFlopAutomatically
        // maybe worth to do it on backend side in future
        if (newState.showdownOutcome && newState.showdownOutcome.processFlopAutomatically) {
            this.handleAutomaticShowdown(newState, prevState, setPlayers, setBetHistory, betHistory)
            await this.animateShowdown(newState, prevState, setBoardCards);
        } else {
            betHistory.calculateBetHistory(newState, false);

            setBetHistory(betHistory);
            setBoardCards(newState.street?.cards);
        }
        setPlayers(GameStateProcessHelper.center_players_by_self(newState.players, newState.playerId));

        return newState;
    }


    private static handleAutomaticShowdown(
        newState: ClientState,
        prevState: ClientState,
        setPlayers: React.Dispatch<React.SetStateAction<Player[] | undefined>>,
        setBetHistory: React.Dispatch<React.SetStateAction<BetHistory>>,
        betHistory: BetHistory
    ) {
        const lastAction = newState.actionHistory[newState.actionHistory.length - 1];
        const player = this.findPlayerById(prevState.players, lastAction.playerId);

        if (player) {
            this.updatePlayerAction(player, lastAction);
        }

        setPlayers(GameStateProcessHelper.center_players_by_self(prevState.players, prevState.playerId));
        betHistory.calculateBetHistory(newState, true);
        setBetHistory(betHistory);
    }

    private static findPlayerById(players: Player[], playerId: number): Player | undefined {
        return players.find((p) => p.userId === playerId);
    }

    private static updatePlayerAction(player: Player, action: any) {
        player.action = action;

        if (action.actionType === ActionType.Call || action.actionType === ActionType.Raise) {
            player.bank -= action.bet;
        }
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

