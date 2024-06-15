import { createBrowserRouter, RouterProvider } from 'react-router-dom';
import GameView from './view/game-view'
import IndexView from './view/index-view'
import Game from './components/game/game';
import { WebSocketContext } from './providers/web-socket-provider';
import { createContext, useContext, useEffect, useRef, useState } from 'react';
import EventEmitter from 'eventemitter3';
import { ResponseMessage, ResponseMessageType, StartGameResponse } from './types/responses';
import { ClientState } from './types/client_state';


const UserContext = createContext<{ id: number | undefined } | null>(null);

export const useUser = () => {
  const context = useContext(UserContext)
  if (!context) {
    throw new Error('useUser must be used within a UserProvider');
  }
  return context;
}

function App() {
  const ws = useRef<WebSocket | null>(null);
  const emitter = useRef<EventEmitter | null>(null);
  const [id, setId] = useState<number>();
  const reff = useRef<HTMLInputElement>(null);

  const addEventListener = (eventName: string, listener: (...args: any[]) => void) => {
    emitter.current?.addListener(eventName, listener);
  }

  const removeEventListener = (eventName: string, listener: (...args: any[]) => void) => {
    emitter.current?.removeListener(eventName, listener);
  }
  // settings  user id for debug 
  const onClick = () => {
    emitter.current = new EventEmitter();
    
    const id = reff.current!.value;
    setId(parseInt(id));
    ws.current = new WebSocket(`ws://127.0.0.1:7878/socket?user_id=${id}`);

    ws.current.onopen = () => { console.log("WebSocket connection established") };
    ws.current.onclose = () => console.log('WebSocket connection closed');
    ws.current.onerror = (error) => console.log('WebSocket error:', error);
    ws.current.onmessage = (event) => {
      (event.data as Blob).arrayBuffer().then((b) => {
        let message = ResponseMessage.fromBinary(new Uint8Array(b));
        switch (message.payloadType) {
          case ResponseMessageType.StartGame: {
            let data = StartGameResponse.fromBinary(message.payload);
            console.log(data);
            break;
          }

          case ResponseMessageType.ClientState: {
            let data = ClientState.fromBinary(message.payload, { readUnknownField: false });
            console.log(data);
            emitter.current?.emit(ResponseMessageType.ClientState.toString(), data);
            break;
          }
        }

      })
    }
  }

  // useEffect(() => {
  //   ws.current = new WebSocket('ws://127.0.0.1:7878/socket?user_id=1');

  //   ws.current.onopen = () => { console.log("WebSocket connection established") };
  //   ws.current.onclose = () => console.log('WebSocket connection closed');
  //   ws.current.onerror = (error) => console.log('WebSocket error:', error);
  //   ws.current.onmessage = (event) => {
  //     (event.data as Blob).arrayBuffer().then((b) => {
  //       let message = ResponseMessage.fromBinary(new Uint8Array(b));
  //       switch (message.payloadType) {
  //         case ResponseMessageType.StartGame: {
  //           let data = StartGameResponse.fromBinary(message.payload);
  //           console.log(data);
  //           break;
  //         }

  //         case ResponseMessageType.ClientState: {
  //           let data = ClientState.fromBinary(message.payload, { readUnknownField: false });
  //           console.log(data);
  //           emitter.emit(ResponseMessageType.ClientState.toString(), data);
  //           break;
  //         }
  //       }

  //     })

  //   }

  //   return () => {
  //     if (ws.current) {
  //       let curr = ws.current;
  //       if (curr.readyState === curr.OPEN) {
  //         curr.close();
  //       } else {
  //         curr.addEventListener('open', () => {
  //           curr.close();
  //         })
  //       }
  //     }

  //   }
  // }, [])





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
      <UserContext.Provider value={{ id }}>
        <RouterProvider router={router}>
        </RouterProvider>
        <input aria-label='set_id' ref={reff}></input>
        <button onClick={() => { onClick() }}>set player id</button>
      </UserContext.Provider>
    </WebSocketContext.Provider>

  );
}
export default App
