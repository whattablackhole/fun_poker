import { createContext, useContext } from 'react';
import WebSocketService from '../services/websocket.service';

  
export const WebSocketContext = createContext<WebSocketService | undefined>(undefined);

export const useWebSocketContext = () => {
    const context = useContext(WebSocketContext)
    if (!context) {
      throw new Error('useWebSocket must be used within a WebSocketProvider');
    }
    return context;
}
