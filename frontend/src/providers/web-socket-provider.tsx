import { MutableRefObject, createContext, useContext } from 'react';



interface WebSocketContextProps {
    connection: MutableRefObject<WebSocket | null> | null;
    addEventListener: (eventName: string, listener: (...args: any[]) => void) => void;
    removeEventListener: (eventName: string, listener: (...args: any[]) => void) => void;
    reconnect: (url: string) => void;
  }
  
export const WebSocketContext = createContext<WebSocketContextProps | undefined>(undefined);



export const useWebSocket = () => {
    const context = useContext(WebSocketContext)
    if (!context) {
      throw new Error('useWebSocket must be used within a WebSocketProvider');
    }
    return context;
}