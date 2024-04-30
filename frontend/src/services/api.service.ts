import { LobbyList } from "../types/lobby";

const localhost = "http://127.0.0.1:7878";
class ApiService {
    public getLobbies(): Promise<LobbyList> {
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
}

export default ApiService;