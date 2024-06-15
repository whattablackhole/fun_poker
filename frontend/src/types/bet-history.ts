import { ClientState } from "./client_state";
import { StreetStatus } from "./game_state";
type PlayerId = number;
type BetAmount = number;

type BetHistoryMap = Map<PlayerId, { [key in StreetStatus]: BetAmount }>;

interface IBetHistory {
    calculateBetHistory(newState: ClientState, automatedShowdown: boolean): void;
    calculateTotalBankOfPrevStreets(prevStreet: StreetStatus): number;
    getBankOnPrevStreet(): number;
    getPlayerBetAmount(playerId: number, street: StreetStatus): number;
}
//TODO: move out from this folder and name this folder... 
class BetHistory implements IBetHistory {
    private _bank_on_prev_street: number = 0;
    private _betHistoryMap: BetHistoryMap = new Map();

    public calculateBetHistory(newState: ClientState, automatedShowdown: boolean) {
        if (!newState.street) {
            return;
        }

        this._betHistoryMap.clear();

        this._bank_on_prev_street = 0;

        newState.actionHistory.forEach((action) => {
            let history = this._betHistoryMap.get(action.playerId) || { 0: 0, 1: 0, 2: 0, 3: 0 };
            history[action.streetStatus] += action.bet;
            this._betHistoryMap.set(action.playerId, history);
        });

        if (newState.street.streetStatus !== 0) {
            this._bank_on_prev_street = this.calculateTotalBankOfPrevStreets(newState.street.streetStatus - 1);
        }

        // NOTE: maybe not needed, rethink
        if (automatedShowdown) {
            this._betHistoryMap.clear();
        }
    }

    public calculateTotalBankOfPrevStreets(prevStreet: StreetStatus): number {
        let totalBank = 0;

        while (prevStreet > -1) {
            for (const [, object] of this._betHistoryMap) {
                totalBank += object[prevStreet];
            }
            --prevStreet;
        }

        return totalBank;
    }

    public getBankOnPrevStreet(): number {
        return this._bank_on_prev_street;
    }

    public getPlayerBetAmount(playerId: number, street?: StreetStatus): number {
        if (street === undefined) {
            return 0;
        }

        return this._betHistoryMap.get(playerId)?.[street] || 0;
    }
}

export default BetHistory;