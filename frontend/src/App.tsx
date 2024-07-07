import { createBrowserRouter, RouterProvider } from "react-router-dom";
import GameView from "./view/game-view";
import IndexView from "./view/index-view";
import Game from "./components/game/game";
import { WebSocketContext } from "./providers/web-socket-provider";
import { useEffect, useRef, useState } from "react";
import EventEmitter from "eventemitter3";
import {
  ResponseMessage,
  ResponseMessageType,
  StartGameResponse,
} from "./types/responses";
import { ClientState } from "./types/client_state";
import { UserContext } from "./providers/user-provider";
import NavigationHeader from "./components/navigation_header/navigation-header";
import { Container } from "@mui/material";
import CreateLobbyDialog from "./components/popups/create-lobby-dialog";
import GoogleSignIn from "./providers/google-signin-provider";
import ApiService from "./services/api.service";
import { User } from "./types/user";

const wsUrl = import.meta.env.VITE_WS_URL;

const initSocket = (url: string, skipConnecitonEstablishment = false) => {
  const ws = useRef<WebSocket | null>(null);
  const emitter = useRef<EventEmitter | null>(null);

  const connect = (url: string) => {
    emitter.current = new EventEmitter();

    ws.current = new WebSocket(url);

    ws.current.onopen = () => {
      console.log("WebSocket connection established");
    };

    ws.current.onclose = () => console.log("WebSocket connection closed");
    ws.current.onerror = (error) => console.log("WebSocket error:", error);
    ws.current.onmessage = (event) => {
      if (!(event.data instanceof Blob)) {
        console.log(event.data);
      }

      (event.data as Blob).arrayBuffer().then((b) => {
        let message = ResponseMessage.fromBinary(new Uint8Array(b));
        switch (message.payloadType) {
          case ResponseMessageType.StartGame: {
            let data = StartGameResponse.fromBinary(message.payload);
            console.log(data);
            break;
          }
          case ResponseMessageType.ClientState: {
            let data = ClientState.fromBinary(message.payload, {
              readUnknownField: false,
            });
            emitter.current?.emit(
              ResponseMessageType.ClientState.toString(),
              data
            );
            break;
          }
        }
      });
    };
  };

  useEffect(() => {
    emitter.current = new EventEmitter();

    if (skipConnecitonEstablishment) {
      return;
    }
    connect(url);

    return () => {
      if (ws.current) {
        let curr = ws.current;
        if (curr.readyState === curr.OPEN) {
          curr.close();
        } else {
          curr.addEventListener("open", () => {
            curr.close();
          });
        }
      }
    };
  }, [url]);

  const addEventListener = (
    eventName: string,
    listener: (...args: any[]) => void
  ) => {
    emitter.current?.addListener(eventName, listener);
  };

  const removeEventListener = (
    eventName: string,
    listener: (...args: any[]) => void
  ) => {
    emitter.current?.removeListener(eventName, listener);
  };

  const reconnect = (newUrl: string) => {
    if (ws.current) {
      ws.current.close();
    }
    connect(newUrl);
  };

  return { addEventListener, removeEventListener, ws, reconnect };
};

const useAuth = () => {
  const [user, setUser] = useState<User | undefined>(undefined);

  useEffect(() => {
    // const loggedInUser = fetchUser();
    // if (loggedInUser) setUser(loggedInUser);
  }, []);

  const login = (user: User) => {
    console.log(user);
    setUser(user);
  };

  const logout = async () => {
    setUser(undefined);
    await ApiService.logout();
  };

  return { user, login, logout };
};

function App() {
  const { user, login, logout } = useAuth();

  const wsRootUrl = wsUrl + "/ws";

  const { addEventListener, removeEventListener, ws, reconnect } = initSocket(
    wsRootUrl,
    !user
  );

  const signInByGoogleHandler = async (user: User) => {
    login(user);
  };

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
              <GoogleSignIn signInHandler={signInByGoogleHandler} />
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
        <UserContext.Provider value={{ user }}>
          <RouterProvider router={router}></RouterProvider>
        </UserContext.Provider>
      </WebSocketContext.Provider>
    </div>
  );
}
export default App;
