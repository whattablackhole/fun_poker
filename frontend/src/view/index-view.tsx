import { useEffect, useRef } from "react";
import NavigationHeader from "../components/navigation_header/navigation-header.tsx";
import LobbiesTable from "../components/navigation_table/lobbies-table.tsx";
import ApiService from "../services/api.service.ts";
import { ClientState } from "../types/client-state.ts";
import { useNavigate } from "react-router-dom";

function IndexView() {
    let lobbyIdInputRef = useRef<HTMLInputElement>(null);
    let playerIdInputRef = useRef<HTMLInputElement>(null);
    let navigate = useNavigate();
    const onClickHandler: React.MouseEventHandler<HTMLButtonElement> = () => {
        ApiService.startGame();
    }
    const onJoinLobbyHandler = () => {
        let lobby_id = Number.parseInt(lobbyIdInputRef.current!.value);
        let player_id = Number.parseInt(playerIdInputRef.current!.value);

        ApiService.establishSocketConnection(lobby_id, player_id);
    }

    useEffect(() => {
        let subscription = ApiService.clientStateObserver.subscribe((v: ClientState) => {
            navigate("/game", { state: v });

        })
        return () => {
            ApiService.clientStateObserver.unsubscribe(subscription);
        }
    }, [navigate])

    return <div style={{ display: "flex", flexDirection: 'column', gap: "100px" }}>
        <NavigationHeader></NavigationHeader>
        <button onClick={onClickHandler}>Start game</button>
        <form>
            <label >Lobby id</label>
            <input ref={lobbyIdInputRef}></input>
        </form>

        <form>
            <label >Player id</label>
            <input ref={playerIdInputRef}></input>
        </form>

        <button onClick={onJoinLobbyHandler}>Join Lobby</button>
        <LobbiesTable></LobbiesTable>
    </div>

}
export default IndexView;