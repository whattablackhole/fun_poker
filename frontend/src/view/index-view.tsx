import NavigationHeader from "../components/navigation_header/navigation-header.tsx";
import LobbiesTable from "../components/navigation_table/lobbies-table.tsx";
import ApiService from "../services/api.service.ts";
import { useNavigate } from "react-router-dom";
import { JoinLobbyRequest } from "../types/requests.ts";
import { useUser } from "../App.tsx";
import CreateLobbyDialog from "../components/popups/create-lobby-dialog.tsx";
import { Button, Container } from "@mui/material";
import GoogleSignIn from "../providers/google-signin-provider.tsx";
import { useWebSocket } from "../providers/web-socket-provider.tsx";

function IndexView() {
    let { id } = useUser();
    let  { reconnect } = useWebSocket();
    let navigate = useNavigate();

    const onJoinLobbyHandler = (lobbyId: number) => {
        if (id) {
            reconnect(`ws://127.0.0.1:8080/join_lobby?lobby_id=${lobbyId}&user_id=${id}`)

            navigate("/table");
            // let request = JoinLobbyRequest.create({ lobbyId, playerId: id });
            // ApiService.joinLobby(request);
        }
    }

    // const headerMenuItemsDefinition: HeaderMenuItemsDefinition = { items: [{ textContent: 'Create New Lobby', type: 'button-popover' }] };

    // const MenuItems = NavigationMenuBuilder.buildMenuItems(headerMenuItemsDefinition);

    return <div style={{ display: "flex", flexDirection: 'column', gap: "100px", background: "linear-gradient(to bottom, #290133, white)" }}>
        <NavigationHeader>
            <Container sx={{ flexDirection: 'row', display: "flex", justifyContent: 'space-between' }}>
                <CreateLobbyDialog></CreateLobbyDialog>
                <div style={{ display: 'flex', alignItems: "flex-end" }}>
                    <GoogleSignIn />
                    {/* <Button size="medium"></Button> */}
                </div>
            </Container>


        </NavigationHeader>
        <Container sx={{ flexDirection: 'column', display: "flex", alignItems:"center" }}>
            <div>
                <img src={"./src/assets/logo_no_background.svg"} width={500} height={500} style={{ alignSelf: "center" }}></img>
                <h1 style={{ alignSelf: "center" }}>Play poker against AI</h1>
            </div>

            <LobbiesTable joinLobbyHandler={onJoinLobbyHandler}></LobbiesTable>
        </Container>

    </div>

}
export default IndexView;