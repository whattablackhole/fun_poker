// @generated by protobuf-ts 2.9.4
// @generated from protobuf file "client-state.proto" (package "client_state", syntax proto3)
// tslint:disable
import type { BinaryWriteOptions } from "@protobuf-ts/runtime";
import type { IBinaryWriter } from "@protobuf-ts/runtime";
import { WireType } from "@protobuf-ts/runtime";
import type { BinaryReadOptions } from "@protobuf-ts/runtime";
import type { IBinaryReader } from "@protobuf-ts/runtime";
import { UnknownFieldHandler } from "@protobuf-ts/runtime";
import type { PartialMessage } from "@protobuf-ts/runtime";
import { reflectionMergePartial } from "@protobuf-ts/runtime";
import { MessageType } from "@protobuf-ts/runtime";
import { Action } from "./player";
import { ShowdownOutcome } from "./game_state";
import { Player } from "./player";
import { GameStatus } from "./game_state";
import { Street } from "./game_state";
import { CardPair } from "./card";
/**
 * @generated from protobuf message client_state.ClientState
 */
export interface ClientState {
    /**
     * @generated from protobuf field: int32 player_id = 1;
     */
    playerId: number;
    /**
     * @generated from protobuf field: card.CardPair cards = 2;
     */
    cards?: CardPair;
    /**
     * @generated from protobuf field: int32 curr_player_id = 3;
     */
    currPlayerId: number;
    /**
     * @generated from protobuf field: int32 curr_button_id = 4;
     */
    currButtonId: number;
    /**
     * @generated from protobuf field: int32 curr_small_blind_id = 5;
     */
    currSmallBlindId: number;
    /**
     * @generated from protobuf field: int32 curr_big_blind_id = 6;
     */
    currBigBlindId: number;
    /**
     * @generated from protobuf field: int32 lobby_id = 7;
     */
    lobbyId: number;
    /**
     * @generated from protobuf field: game_state.Street street = 8;
     */
    street?: Street;
    /**
     * @generated from protobuf field: game_state.GameStatus game_status = 9;
     */
    gameStatus: GameStatus;
    /**
     * @generated from protobuf field: repeated player.Player players = 10;
     */
    players: Player[];
    /**
     * @generated from protobuf field: optional game_state.ShowdownOutcome showdown_outcome = 11;
     */
    showdownOutcome?: ShowdownOutcome;
    /**
     * @generated from protobuf field: int32 amount_to_call = 12;
     */
    amountToCall: number;
    /**
     * @generated from protobuf field: int32 min_amount_to_raise = 13;
     */
    minAmountToRaise: number;
    /**
     * @generated from protobuf field: bool can_raise = 14;
     */
    canRaise: boolean;
    /**
     * @generated from protobuf field: repeated player.Action action_history = 15;
     */
    actionHistory: Action[];
}
// @generated message type with reflection information, may provide speed optimized methods
class ClientState$Type extends MessageType<ClientState> {
    constructor() {
        super("client_state.ClientState", [
            { no: 1, name: "player_id", kind: "scalar", T: 5 /*ScalarType.INT32*/ },
            { no: 2, name: "cards", kind: "message", T: () => CardPair },
            { no: 3, name: "curr_player_id", kind: "scalar", T: 5 /*ScalarType.INT32*/ },
            { no: 4, name: "curr_button_id", kind: "scalar", T: 5 /*ScalarType.INT32*/ },
            { no: 5, name: "curr_small_blind_id", kind: "scalar", T: 5 /*ScalarType.INT32*/ },
            { no: 6, name: "curr_big_blind_id", kind: "scalar", T: 5 /*ScalarType.INT32*/ },
            { no: 7, name: "lobby_id", kind: "scalar", T: 5 /*ScalarType.INT32*/ },
            { no: 8, name: "street", kind: "message", T: () => Street },
            { no: 9, name: "game_status", kind: "enum", T: () => ["game_state.GameStatus", GameStatus] },
            { no: 10, name: "players", kind: "message", repeat: 1 /*RepeatType.PACKED*/, T: () => Player },
            { no: 11, name: "showdown_outcome", kind: "message", T: () => ShowdownOutcome },
            { no: 12, name: "amount_to_call", kind: "scalar", T: 5 /*ScalarType.INT32*/ },
            { no: 13, name: "min_amount_to_raise", kind: "scalar", T: 5 /*ScalarType.INT32*/ },
            { no: 14, name: "can_raise", kind: "scalar", T: 8 /*ScalarType.BOOL*/ },
            { no: 15, name: "action_history", kind: "message", repeat: 1 /*RepeatType.PACKED*/, T: () => Action }
        ]);
    }
    create(value?: PartialMessage<ClientState>): ClientState {
        const message = globalThis.Object.create((this.messagePrototype!));
        message.playerId = 0;
        message.currPlayerId = 0;
        message.currButtonId = 0;
        message.currSmallBlindId = 0;
        message.currBigBlindId = 0;
        message.lobbyId = 0;
        message.gameStatus = 0;
        message.players = [];
        message.amountToCall = 0;
        message.minAmountToRaise = 0;
        message.canRaise = false;
        message.actionHistory = [];
        if (value !== undefined)
            reflectionMergePartial<ClientState>(this, message, value);
        return message;
    }
    internalBinaryRead(reader: IBinaryReader, length: number, options: BinaryReadOptions, target?: ClientState): ClientState {
        let message = target ?? this.create(), end = reader.pos + length;
        while (reader.pos < end) {
            let [fieldNo, wireType] = reader.tag();
            switch (fieldNo) {
                case /* int32 player_id */ 1:
                    message.playerId = reader.int32();
                    break;
                case /* card.CardPair cards */ 2:
                    message.cards = CardPair.internalBinaryRead(reader, reader.uint32(), options, message.cards);
                    break;
                case /* int32 curr_player_id */ 3:
                    message.currPlayerId = reader.int32();
                    break;
                case /* int32 curr_button_id */ 4:
                    message.currButtonId = reader.int32();
                    break;
                case /* int32 curr_small_blind_id */ 5:
                    message.currSmallBlindId = reader.int32();
                    break;
                case /* int32 curr_big_blind_id */ 6:
                    message.currBigBlindId = reader.int32();
                    break;
                case /* int32 lobby_id */ 7:
                    message.lobbyId = reader.int32();
                    break;
                case /* game_state.Street street */ 8:
                    message.street = Street.internalBinaryRead(reader, reader.uint32(), options, message.street);
                    break;
                case /* game_state.GameStatus game_status */ 9:
                    message.gameStatus = reader.int32();
                    break;
                case /* repeated player.Player players */ 10:
                    message.players.push(Player.internalBinaryRead(reader, reader.uint32(), options));
                    break;
                case /* optional game_state.ShowdownOutcome showdown_outcome */ 11:
                    message.showdownOutcome = ShowdownOutcome.internalBinaryRead(reader, reader.uint32(), options, message.showdownOutcome);
                    break;
                case /* int32 amount_to_call */ 12:
                    message.amountToCall = reader.int32();
                    break;
                case /* int32 min_amount_to_raise */ 13:
                    message.minAmountToRaise = reader.int32();
                    break;
                case /* bool can_raise */ 14:
                    message.canRaise = reader.bool();
                    break;
                case /* repeated player.Action action_history */ 15:
                    message.actionHistory.push(Action.internalBinaryRead(reader, reader.uint32(), options));
                    break;
                default:
                    let u = options.readUnknownField;
                    if (u === "throw")
                        throw new globalThis.Error(`Unknown field ${fieldNo} (wire type ${wireType}) for ${this.typeName}`);
                    let d = reader.skip(wireType);
                    if (u !== false)
                        (u === true ? UnknownFieldHandler.onRead : u)(this.typeName, message, fieldNo, wireType, d);
            }
        }
        return message;
    }
    internalBinaryWrite(message: ClientState, writer: IBinaryWriter, options: BinaryWriteOptions): IBinaryWriter {
        /* int32 player_id = 1; */
        if (message.playerId !== 0)
            writer.tag(1, WireType.Varint).int32(message.playerId);
        /* card.CardPair cards = 2; */
        if (message.cards)
            CardPair.internalBinaryWrite(message.cards, writer.tag(2, WireType.LengthDelimited).fork(), options).join();
        /* int32 curr_player_id = 3; */
        if (message.currPlayerId !== 0)
            writer.tag(3, WireType.Varint).int32(message.currPlayerId);
        /* int32 curr_button_id = 4; */
        if (message.currButtonId !== 0)
            writer.tag(4, WireType.Varint).int32(message.currButtonId);
        /* int32 curr_small_blind_id = 5; */
        if (message.currSmallBlindId !== 0)
            writer.tag(5, WireType.Varint).int32(message.currSmallBlindId);
        /* int32 curr_big_blind_id = 6; */
        if (message.currBigBlindId !== 0)
            writer.tag(6, WireType.Varint).int32(message.currBigBlindId);
        /* int32 lobby_id = 7; */
        if (message.lobbyId !== 0)
            writer.tag(7, WireType.Varint).int32(message.lobbyId);
        /* game_state.Street street = 8; */
        if (message.street)
            Street.internalBinaryWrite(message.street, writer.tag(8, WireType.LengthDelimited).fork(), options).join();
        /* game_state.GameStatus game_status = 9; */
        if (message.gameStatus !== 0)
            writer.tag(9, WireType.Varint).int32(message.gameStatus);
        /* repeated player.Player players = 10; */
        for (let i = 0; i < message.players.length; i++)
            Player.internalBinaryWrite(message.players[i], writer.tag(10, WireType.LengthDelimited).fork(), options).join();
        /* optional game_state.ShowdownOutcome showdown_outcome = 11; */
        if (message.showdownOutcome)
            ShowdownOutcome.internalBinaryWrite(message.showdownOutcome, writer.tag(11, WireType.LengthDelimited).fork(), options).join();
        /* int32 amount_to_call = 12; */
        if (message.amountToCall !== 0)
            writer.tag(12, WireType.Varint).int32(message.amountToCall);
        /* int32 min_amount_to_raise = 13; */
        if (message.minAmountToRaise !== 0)
            writer.tag(13, WireType.Varint).int32(message.minAmountToRaise);
        /* bool can_raise = 14; */
        if (message.canRaise !== false)
            writer.tag(14, WireType.Varint).bool(message.canRaise);
        /* repeated player.Action action_history = 15; */
        for (let i = 0; i < message.actionHistory.length; i++)
            Action.internalBinaryWrite(message.actionHistory[i], writer.tag(15, WireType.LengthDelimited).fork(), options).join();
        let u = options.writeUnknownFields;
        if (u !== false)
            (u == true ? UnknownFieldHandler.onWrite : u)(this.typeName, message, writer);
        return writer;
    }
}
/**
 * @generated MessageType for protobuf message client_state.ClientState
 */
export const ClientState = new ClientState$Type();
