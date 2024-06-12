import { useEffect, useRef } from "react";
import NavigationHeader from "../components/navigation_header/navigation-header.tsx";
import LobbiesTable from "../components/navigation_table/lobbies-table.tsx";
import ApiService from "../services/api.service.ts";
import { ClientState } from "../types/client-state.ts";
import { useNavigate } from "react-router-dom";
import { useWebSocket } from "../providers/web-socket-provider.tsx";
import { JoinLobbyRequest, StartGameRequest } from "../types/requests.ts";

function IndexView() {
    // const { connection, addEventListener, removeEventListener } = useWebSocket();

    
    let lobbyIdInputRef = useRef<HTMLInputElement>(null);
    let playerIdInputRef = useRef<HTMLInputElement>(null);
    let navigate = useNavigate();
    
    const onClickHandler: React.MouseEventHandler<HTMLButtonElement> = () => {
        // navigate("/table");
        let request = StartGameRequest.create({lobbyId:1, playerId: 1})
        ApiService.startGame(request);
    }

    const onJoinLobbyHandler = () => {
        let lobbyId = Number.parseInt(lobbyIdInputRef.current!.value);
        let playerId = Number.parseInt(playerIdInputRef.current!.value);
        
        let request = JoinLobbyRequest.create({ lobbyId, playerId });
        ApiService.joinLobby(request);
    }


    // useEffect(() => {
    //     let subscription = ApiService.clientStateObserver.subscribe((v: ClientState) => {
    //         // navigate("/game", { state: v });
            
    //     })
    //     return () => {
    //         ApiService.clientStateObserver.unsubscribe(subscription);
    //     }
    // }, [navigate])

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