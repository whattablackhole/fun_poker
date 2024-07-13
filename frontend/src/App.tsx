import { createBrowserRouter, RouterProvider } from "react-router-dom";
import IndexView from "./view/index-view";
import Game from "./components/game/game";
import { WebSocketContext } from "./contexts/websocket-context";
import { UserContext } from "./contexts/user-context";
import { useAuth } from "./hooks/useAuth";
import Layout from "./components/layout/layout";
import WebSocketService from "./services/websocket.service";

function App() {
  const { user, login, logout } = useAuth();
  
  const websocketService = new WebSocketService();

  const router = createBrowserRouter([
    {
      path: "/",
      element: <Layout login={login} logout={logout} />,
      children: [
        {
          path: "/",
          Component: IndexView,
        },
      ],
    },
    {
      path: "/table",
      Component: Game,
    },
  ]);
  return (
    <div style={{ background: "linear-gradient(to bottom, #290133, white)" }}>
      <UserContext.Provider value={{ user }}>
        <WebSocketContext.Provider
          value={websocketService}
        >
          <RouterProvider router={router}></RouterProvider>
        </WebSocketContext.Provider>
      </UserContext.Provider>
    </div>
  );
}
export default App;
