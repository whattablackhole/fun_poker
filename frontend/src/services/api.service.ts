import { PlayerPayload } from "../types";
import { ClientState } from "../types/client-state";
import { LobbyList } from "../types/lobby";
import { CreateLobbyRequest, JoinLobbyRequest, StartGameRequest } from "../types/requests";

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
    // TODO: remove
    // public static clientStateObserver: { isRunning: boolean, queue: any[], cb: { callback: Function, id: number }[], subscriptionIdCounter: number, subscribe: Function, unsubscribe: Function, next: Function } = {
    //     cb: [],
    //     subscriptionIdCounter: 0,
    //     queue: [],
    //     isRunning: false,
    //     subscribe: function (cb: Function) {
    //         const subscriptionId = this.subscriptionIdCounter++;
    //         this.cb.push({ id: subscriptionId, callback: cb });
    //         return { id: subscriptionId };
    //     },
    //     unsubscribe: function (subscription: { id: number }) {
    //         const index = this.cb.findIndex(sub => sub.id === subscription.id);
    //         if (index !== -1) {
    //             this.cb.splice(index, 1);
    //         }
    //     },
    //     next: async function (v: any) {
    //         if (this.isRunning) {
    //             this.queue.push(v);
    //             return;
    //         }
    //         this.isRunning = true;
    //         let promises = this.cb.map(sub => new Promise(async (resolve: Function) => {
    //             try {
    //                 await sub.callback(v);
    //                 resolve();
    //             } catch (error) {
    //                 console.error('Error processing value:', error);
    //                 resolve();
    //             }
    //         }));
    //         await Promise.all(promises);
    //         this.isRunning = false;
    //         if (this.queue.length > 0) {
    //             const nextValue = this.queue.shift();
    //             await this.next(nextValue);
    //         }
    //     }
    // };
    // TODO: remove
    // private static socket: WebSocket | null = null;

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

    public static joinLobby(request: JoinLobbyRequest) {
        return fetch(`${localhost}/joinLobby`, { method: "POST", body: JoinLobbyRequest.toBinary(request) }).then((response) => {
            return response.body?.getReader().read().then((s) => {
                if (s.value) {
                    console.log(s);
                }

            });
        })
    }
    // TODO: remove
    // public static sendMessage(v: PlayerPayload) {
    //     if (this.socket) {
    //         this.socket.send(PlayerPayload.toBinary(v));
    //     }
    // }

    // public static establishSocketConnection(lobbyId: number, playerId: number) {
    //     let socket = new WebSocket('ws://127.0.0.1:7878/socket');
    //     socket.onopen = (ev) => {
    //         this.socket = socket;
    //         console.log("connection establshed");
    //     }

    //     socket.onmessage = (msg) => {
    //         let blob = msg.data as Blob;
    //         blob.arrayBuffer().then((b) => {
    //             let state = ClientState.fromBinary(new Uint8Array(b));
    //             // console.log(state);
    //             this.clientStateObserver.next(state);
    //         })
    //     }

    // }

    public static startGame(request: StartGameRequest) {
        fetch(`${localhost}/startGame`, { method: "POST", body: StartGameRequest.toBinary(request) }).then();
    }

    public static createLobby(request: CreateLobbyRequest) {
        fetch(`${localhost}/createLobby`, { method: "POST", body: CreateLobbyRequest.toBinary(request) }).then();
    }
}

export default ApiService;