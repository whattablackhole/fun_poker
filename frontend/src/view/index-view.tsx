import NavigationHeader from "../components/navigation_header/navigation-header.tsx";
import LobbiesTable from "../components/navigation_table/lobbies-table.tsx";
import ApiService from "../services/api.service.ts";
import { useNavigate } from "react-router-dom";
import { JoinLobbyRequest } from "../types/requests.ts";
import { useUser } from "../App.tsx";

function IndexView() {
    let { id } = useUser();

    let navigate = useNavigate();

    const onJoinLobbyHandler = (lobbyId: number) => {
        if (id) {
            navigate("/table");
            let request = JoinLobbyRequest.create({ lobbyId, playerId: id });
            ApiService.joinLobby(request);
        }
    }

    return <div style={{ display: "flex", flexDirection: 'column', gap: "100px" }}>
        <NavigationHeader></NavigationHeader>
        <LobbiesTable joinLobbyHandler={onJoinLobbyHandler}></LobbiesTable>
    </div>

}
export default IndexView;