import { LobbyList } from "../types/lobby";

const localhost = "http://127.0.0.1:7878";
class ApiService {
    public getLobbies() {
        fetch(localhost, { method: "GET" }).then((response) => {
            response.body?.getReader().read().then((s) => {
                if (s.value) {
                    let result = LobbyList.fromBinary(new Uint8Array(s.value.buffer));
                    console.log(result);
                }
            })
        })
    }
}

export default ApiService;