import { createBrowserRouter, RouterProvider } from 'react-router-dom';
import GameView from './view/game-view'
import IndexView from './view/index-view'
import Game from './components/game/game';
import { WebSocketContext } from './providers/web-socket-provider';
import { useEffect, useRef } from 'react';
import EventEmitter from 'eventemitter3';


function App() {
  const ws = useRef<WebSocket | null>(null);
  const emitter = new EventEmitter();

  useEffect(() => {
    let connected = false;
    ws.current = new WebSocket('ws://127.0.0.1:7878/socket?user_id=1');
    ws.current.onopen = () => { connected = true; console.log("WebSocket connection established") };
    ws.current.onclose = () => console.log('WebSocket connection closed');
    ws.current.onerror = (error) => console.log('WebSocket error:', error);
    ws.current.onmessage = (event) => emitter.emit(event.data.eventName, event);

    return () => {
      if (ws.current) {
        let curr = ws.current;
        if (curr.readyState === curr.OPEN) {
          curr.close();
        } else {
          curr.addEventListener('open', () => {
            curr.close();
          })
        }
      }

    }
  }, [])

  const addEventListener = (eventName: string, listener: (...args: any[]) => void) => {
    emitter.addListener(eventName, listener);
  }

  const removeEventListener = (eventName: string, listener: (...args: any[]) => void) => {
    emitter.removeListener(eventName, listener);
  }




  const router = createBrowserRouter([
    {
      path: "/",
      Component: IndexView
    },
    {
      path: "/new-lobby",
      element: <div>Not implemented yet</div>,
    },
    {
      path: "/game",
      Component: GameView
    },
    {
      path: "/table",
      Component: Game
    }
  ]);
  return (
    <WebSocketContext.Provider value={{ addEventListener, removeEventListener, connection: ws.current }}>
      <RouterProvider router={router}>
      </RouterProvider>
    </WebSocketContext.Provider>

  );
}
export default App
