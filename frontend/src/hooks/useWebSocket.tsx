import EventEmitter from "eventemitter3";
import { useRef } from "react";
import {
  ResponseMessage,
  ResponseMessageType,
  StartGameResponse,
} from "../types/responses";
import { ClientState } from "../types";

export const useWebSocket = () => {
  const ws = useRef<WebSocket | null>(null);

  const emitter = useRef<EventEmitter | null>(null);

  const connect = async (url: string) => {
    if (ws.current && ws.current.readyState === ws.current.OPEN) {
      ws.current.close();
    }
    
    emitter.current = new EventEmitter();

    ws.current = new WebSocket(url);

    ws.current.onopen = () => {
      console.log("WebSocket connection established");
    };

    ws.current.onclose = (e) => {
      console.log("WebSocket connection closed");
      emitter.current?.emit(
        CloseEvent.name,
        e.reason
      );
    } 
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

  return { addEventListener, removeEventListener, ws, connect };
};
