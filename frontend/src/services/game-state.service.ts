import GameStateProcessHelper from "../helpers/game-state-helper";
import BetHistory from "../types/bet-history";
import { Card } from "../types/card";
import { ClientState } from "../types/client_state";
import { PlayerCards, StreetStatus, Winner } from "../types/game_state";
import { Player } from "../types/player";

class GameStateService {
  static async processNewState(
    newState: ClientState,
    betHistory: BetHistory,
    setBoardCards: React.Dispatch<React.SetStateAction<Card[] | undefined>>,
    setBetHistory: React.Dispatch<React.SetStateAction<BetHistory>>,
    setPlayers: React.Dispatch<React.SetStateAction<Player[] | undefined>>,
    setWinners: React.Dispatch<React.SetStateAction<Winner[] | undefined>>
  ) {
    // TODO: figure out how to set actual player banks on processFlopAutomatically
    // maybe worth to do it on backend side in future

    betHistory.calculateBetHistory(newState, !!newState.showdownOutcome);
    setBetHistory(betHistory);

    if (newState.showdownOutcome) {
      setPlayers((players) => {
        return this.updatePlayersCards(
          players,
          newState.showdownOutcome?.playersCards
        );
      });

      if (newState.showdownOutcome.processFlopAutomatically) {
        await this.animateShowdown(newState, setBoardCards);
      }

      await new Promise((resolve) => setTimeout(resolve, 2000));

      setWinners(newState.showdownOutcome.winners);

      await new Promise((resolve) => setTimeout(resolve, 4000));

      setWinners(undefined);
    } else {
      setBoardCards(newState.street?.cards);
      this.setupPlayers(newState, setPlayers);
    }

    return newState;
  }

  private static updatePlayersCards(players?: Player[], cards?: PlayerCards[]) {
    cards?.forEach((pc) => {
      let player = players?.find((p) => p.userId == pc.playerId);

      if (player && pc.cards && player.cards === undefined) {
        player.cards = pc.cards;
      }
    });

    return players;
  }

  private static setupPlayers(
    state: ClientState,
    setPlayers: React.Dispatch<React.SetStateAction<Player[] | undefined>>
  ) {
    let selfId = state.playerId;

    let player = state.players.find((p) => p.userId == selfId);

    if (player && state.cards) {
      player.cards = state.cards;
    }

    setPlayers(
      GameStateProcessHelper.center_players_by_self(
        state.players,
        state.playerId
      )
    );
  }

  private static getCurrentCardPositionFromStreetStatus(
    streetStatus?: StreetStatus
  ): number {
    let position = 0;

    if (streetStatus === undefined) {
      return position;
    }

    switch (streetStatus) {
      case StreetStatus.Preflop:
        break;
      case StreetStatus.Flop:
        position = 3;
        break;
      case StreetStatus.Turn:
        position = 4;
        break;
      case StreetStatus.River:
        position = 5;
        break;
      default:
        break;
    }

    return position;
  }
  static async animateShowdown(
    newState: ClientState,
    setBoardCards: React.Dispatch<React.SetStateAction<Card[] | undefined>>
  ) {
    const startPosition = this.getCurrentCardPositionFromStreetStatus(
      newState.showdownOutcome?.streetHistory?.startingStreet
    );
    if (newState.showdownOutcome) {
      let cardsToShow = [
        ...newState.showdownOutcome.streetHistory!.finalBoard!.cards.slice(
          startPosition
        ),
      ];

      for (let c of cardsToShow) {
        await this.setBoardCardsWithDelay(1000, setBoardCards, c);
      }
    }
  }

  static async setBoardCardsWithDelay(
    delay: number,
    setBoardCards: React.Dispatch<React.SetStateAction<Card[] | undefined>>,
    card: Card
  ) {
    return new Promise<void>((resolve) => {
      setTimeout(() => {
        setBoardCards((state) => {
          return [...(state ?? []), card];
        });
        resolve();
      }, delay);
    });
  }
}

export default GameStateService;
