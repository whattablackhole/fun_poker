import { ClientState } from "./client-state";
import { StreetStatus } from "./game_state";
type PlayerId = number;
type BetAmount = number;

type BetHistoryMap = Map<PlayerId, { [key in StreetStatus]: BetAmount }>;

interface IBetHistory {
    calculateBetHistory(newState: ClientState, automatedShowdown: boolean): this;
    calculateTotalBankOfPrevStreets(this: BetHistoryMap, prevStreet: StreetStatus): number;
    getBankOnPrevStreet(): number;
    getPlayerthis(): BetHistoryMap;
}
//TODO: move out from this folder and name this folder... 
class BetHistory implements IBetHistory {
    private _bank_on_prev_street: number = 0;
    private _betHistoryMap: BetHistoryMap = new Map();

    public calculateBetHistory(newState: ClientState, automatedShowdown: boolean): this {
        this._betHistoryMap.clear();
        this._bank_on_prev_street = 0;
        newState.actionHistory.forEach((action) => {
            let history = this._betHistoryMap.get(action.playerId) || { 0: 0, 1: 0, 2: 0, 3: 0 };
            history[action.streetStatus] += action.bet;
            this._betHistoryMap.set(action.playerId, history);
        });
        if (newState.street?.streetStatus !== 0) {
            this._bank_on_prev_street = this.calculateTotalBankOfPrevStreets(newState.street!.streetStatus - 1);
        }
        // NOTE: maybe not needed, rethink
        if (automatedShowdown) {
            this._betHistoryMap.clear();
        }
        return this;
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

    public getPlayerthis(): BetHistoryMap {
        return this._betHistoryMap;
    }
}

export default BetHistory;