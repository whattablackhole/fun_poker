import LobbiesTable from "../components/navigation_table/lobbies-table.tsx";
import { useNavigate } from "react-router-dom";
import { Container } from "@mui/material";
import { useWebSocketContext } from "../contexts/websocket-context.tsx";
import CreateTempUserDialog from "../components/popups/temporal-user-creation-dialog.tsx";
import { useState } from "react";
import { useUserContext } from "../contexts/user-context.tsx";
import ApiService from "../services/api.service.ts";

const wsUrl = import.meta.env.VITE_WS_URL;

function IndexView() {
  let { user } = useUserContext();
  let { reconnect } = useWebSocketContext();
  let navigate = useNavigate();
  let [openCreateUser, setOpenCreateUser] = useState(false);
  const [pendingLobbyId, setPendingLobbyId] = useState<number | null>(null);

  const onJoinLobbyHandler = (lobbyId: number) => {
    if (user?.id) {
      reconnect(`${wsUrl}/join_lobby?lobby_id=${lobbyId}`);
      navigate("/table");
    } else {
      setPendingLobbyId(lobbyId);
      setOpenCreateUser(true);
    }
  };

  const onClose = () => {
    setOpenCreateUser(false);
  };

  const createTempUserHandler = (userName: string, countryCode: string) => {
    setOpenCreateUser(false);

    if (pendingLobbyId) {
      ApiService.fetchTempAccessToken(userName, countryCode).then((r) => {
        if (r.ok) {
          reconnect(`${wsUrl}/join_lobby?lobby_id=${pendingLobbyId}`);
          navigate("/table");
        }
      });
    }
  };

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        gap: "100px",
      }}
    >
      <Container
        sx={{ flexDirection: "column", display: "flex", alignItems: "center" }}
      >
        <div>
          <img
            src={"/src/assets/logo_no_background.svg"}
            width={500}
            height={500}
            style={{ alignSelf: "center" }}
          ></img>
          <h1 style={{ alignSelf: "center" }}>Play poker against AI</h1>
        </div>

        <LobbiesTable joinLobbyHandler={onJoinLobbyHandler}></LobbiesTable>
      </Container>
      {openCreateUser ? (
        <CreateTempUserDialog
          submitHandler={createTempUserHandler}
          onCloseHandler={onClose}
          open={openCreateUser}
        ></CreateTempUserDialog>
      ) : null}
    </div>
  );
}
export default IndexView;
