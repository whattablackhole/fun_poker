import { LobbyList } from "../types/lobby";
import { CreateLobbyRequest, JoinLobbyRequest, SpawnBotRequest, StartGameRequest } from "../types/requests";


const apiUrl = import.meta.env.VITE_API_URL;

class ApiService {
    public static getLobbies(): Promise<LobbyList> {
        return fetch(`${apiUrl}/getLobbies`, { method: "GET" }).then((response) => {
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
        return fetch(`${apiUrl}/joinLobby`, { method: "POST", body: JoinLobbyRequest.toBinary(request) }).then((response) => {
            return response.body?.getReader().read().then();
        })
    }

    public static startGame(request: StartGameRequest) {
        fetch(`${apiUrl}/startGame`, { method: "POST", body: StartGameRequest.toBinary(request) }).then();
    }

    public static createLobby(request: CreateLobbyRequest) {
        fetch(`${apiUrl}/createLobby`, { method: "POST", body: CreateLobbyRequest.toBinary(request) }).then();
    }

    public static spawnBot(request: SpawnBotRequest) {
        fetch(`${apiUrl}/spawnAIBot`, { method: "POST", body: SpawnBotRequest.toBinary(request) }).then();
    }
}

export default ApiService;