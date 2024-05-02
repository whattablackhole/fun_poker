import { PlayerPayload } from "../types";
import { CardPair, CardSuit } from "../types/card";
import { ClientState } from "../types/client-state";
import { JoinLobbyRequest } from "../types/join-lobby.request";
import { LobbyList } from "../types/lobby";

const localhost = "http://127.0.0.1:7878";

// for debug
function getEnumString(enumObject: any, value: any) {
    for (let key in enumObject) {
        if (enumObject[key] === value) {
            return key;
        }
    }
    return "Unknown";
}

class ApiService {
    public static clientStateObserver: { cb: { callback: Function, id: number }[], subscriptionIdCounter: number, subscribe: Function, unsubscribe: Function, next: Function } = {
        cb: [],
        subscriptionIdCounter: 0,
        subscribe: function (cb: Function) {
            const subscriptionId = this.subscriptionIdCounter++;
            this.cb.push({ id: subscriptionId, callback: cb });
            return { id: subscriptionId };
        },
        unsubscribe: function (subscription: { id: number }) {
            const index = this.cb.findIndex(sub => sub.id === subscription.id);
            if (index !== -1) {
                this.cb.splice(index, 1);
            }
        },
        next: function (v: any) {
            this.cb.forEach(sub => sub.callback(v));
        }
    };
    private static socket: WebSocket | null = null;

    public static getLobbies(): Promise<LobbyList> {
        return fetch(`${localhost}/getLobbies`, { method: "GET" }).then((response) => {
            return response.body?.getReader().read().then((s) => {
                if (s.value) {
                    let result = LobbyList.fromBinary(new Uint8Array(s.value.buffer));
                    return result;
                }
                return { list: [] } as LobbyList;
            }) ?? { list: [] } as LobbyList;
        })
    }
    public static sendMessage(v: PlayerPayload) {
        if (this.socket) {
            this.socket.send(PlayerPayload.toBinary(v));
        }
    }
    public static establishSocketConnection(lobbyId: number, playerId: number) {
        let socket = new WebSocket('ws://127.0.0.1:7878/socket');
        socket.onopen = (ev) => {
            this.socket = socket;
            console.log("connection establshed");
            let request = JoinLobbyRequest.create({ lobbyId, playerId });
            socket.send(JoinLobbyRequest.toBinary(request));
        }

        socket.onmessage = (msg) => {
            let blob = msg.data as Blob;
            blob.arrayBuffer().then((b) => {
                let state = ClientState.fromBinary(new Uint8Array(b));
                console.log(state);
                this.clientStateObserver.next(state);
            })
        }

    }

    public static startGame() {
        fetch(`${localhost}/gameStart`, { method: "GET" }).then();
    }
}

export default ApiService;