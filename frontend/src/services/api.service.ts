import { LobbyList } from "../types/lobby";

const localhost = "http://127.0.0.1:7878";
class ApiService {
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
    public static establishSocketConnection() {
        let socket = new WebSocket('ws://127.0.0.1:7878/socket');
        socket.onopen = (ev)=>{
                console.log("connection establshed");
        }

        socket.onmessage = (msg)=>{
            console.log(msg);
        }
       
    }

    public static startGame() {
         fetch(`${localhost}/sendMessageToAll`, { method: "GET" }).then();
    }
}

export default ApiService;