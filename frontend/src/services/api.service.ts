import { LobbyList } from "../types/lobby";
import { CreateLobbyRequest, JoinLobbyRequest, SpawnBotRequest, StartGameRequest } from "../types/requests";

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

    public static joinLobby(request: JoinLobbyRequest) {
        return fetch(`${localhost}/joinLobby`, { method: "POST", body: JoinLobbyRequest.toBinary(request) }).then((response) => {
            return response.body?.getReader().read().then();
        })
    }

    public static startGame(request: StartGameRequest) {
        fetch(`${localhost}/startGame`, { method: "POST", body: StartGameRequest.toBinary(request) }).then();
    }

    public static createLobby(request: CreateLobbyRequest) {
        fetch(`${localhost}/createLobby`, { method: "POST", body: CreateLobbyRequest.toBinary(request) }).then();
    }

    public static spawnBot(request: SpawnBotRequest) {
        fetch(`${localhost}/spawnAIBot`, { method: "POST", body: SpawnBotRequest.toBinary(request) }).then();
    }
}

export default ApiService;