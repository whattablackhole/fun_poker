import EventEmitter from "eventemitter3";
import { useEffect, useRef } from "react";
import { ResponseMessage, ResponseMessageType, StartGameResponse } from "../types/responses";
import { ClientState } from "../types";

export const useWebSocket = (url: string, skipInit = false) => {
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
  
      if (skipInit) {
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