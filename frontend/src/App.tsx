import { createBrowserRouter, RouterProvider } from "react-router-dom";
import GameView from "./view/game-view";
import IndexView from "./view/index-view";
import Game from "./components/game/game";
import { WebSocketContext } from "./contexts/websocket-context";
import NavigationHeader from "./components/navigation_header/navigation-header";
import { Container } from "@mui/material";
import CreateLobbyDialog from "./components/popups/create-lobby-dialog";
import { UserContext } from "./contexts/user-context";
import { useAuth } from "./hooks/useAuth";
import { useWebSocket } from "./hooks/useWebSocket";
import GoogleSignIn from "./components/googel-signin/google-signin";

const wsUrl = import.meta.env.VITE_WS_URL;

function App() {
  const { user, login, logout } = useAuth();

  const wsRootUrl = wsUrl + "/ws";

  const { addEventListener, removeEventListener, ws, reconnect } = useWebSocket(
    wsRootUrl,
    !user
  );

  const router = createBrowserRouter([
    {
      path: "/",
      Component: IndexView,
    },
    {
      path: "/new-lobby",
      element: <div>Not implemented yet</div>,
    },
    {
      path: "/game",
      Component: GameView,
    },
    {
      path: "/table",
      Component: Game,
    },
  ]);
  return (
    <div style={{ background: "linear-gradient(to bottom, #290133, white)" }}>
      <UserContext.Provider value={{ user }}>
      <NavigationHeader>
        <Container
          sx={{
            flexDirection: "row",
            display: "flex",
            justifyContent: "space-between",
          }}
        >
          <CreateLobbyDialog></CreateLobbyDialog>
          <div style={{ display: "flex", alignItems: "flex-end" }}>
            {user ? (
              <div
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: "15px",
                }}
              >
                <div>{user.name}</div>
                <button onClick={logout}>Logout</button>
              </div>
            ) : (
              <GoogleSignIn signInHandler={login} />
            )}
          </div>
        </Container>
      </NavigationHeader>
      <WebSocketContext.Provider
        value={{
          addEventListener,
          removeEventListener,
          connection: ws,
          reconnect,
        }}
      >
          <RouterProvider router={router}></RouterProvider>
      </WebSocketContext.Provider>
      </UserContext.Provider>
    </div>
  );
}
export default App;
