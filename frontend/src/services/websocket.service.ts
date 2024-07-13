import EventEmitter from "eventemitter3";
import {
  ResponseMessage,
  ResponseMessageType,
  StartGameResponse,
} from "../types/responses";
import { ClientState } from "../types";

export default class WebSocketService {
  private ws: WebSocket | null = null;
  private emitter: EventEmitter | null = null;
  private wsPromise: Promise<void> | null = null;

  public reconnect(url: string): Promise<void> {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.ws.close();
    }
    return this.connect(url);
  }

  public disconnect() {
    if (this.ws) {
      this.ws.onclose = null;
      this.ws.close();
      this.ws = null;
    }

    if (this.emitter) {
      this.emitter.emit("Disconnecting");
      this.emitter = null;
    }

    this.wsPromise = null;
  }

  public sendMessage(message: Uint8Array) {
    if (this.ws) {
      this.ws.send(message);
    } else {
      throw Error("websocket is not initialized");
    }
  }

  public connect(url: string): Promise<void> {
    if (this.wsPromise) {
      return this.wsPromise;
    }

    this.wsPromise = new Promise((resolve, reject) => {
      this.emitter = new EventEmitter();
      this.ws = new WebSocket(url);

      this.ws.onopen = () => {
        console.log("WebSocket connection established");
        this.wsPromise = null;
        resolve();
      };

      this.ws.onclose = (e) => {
        console.log("WebSocket connection closed");
        this.wsPromise = null;
        this.ws = null;
        if (this.emitter) {
          this.emitter.emit("close", e.reason);
        }
      };

      this.ws.onerror = (error) => {
        console.log("WebSocket error:", error);
        this.wsPromise = null;
        this.ws = null;
        reject(error);
      };

      this.ws.onmessage = async (event) => {
        try {
          let message: ResponseMessage;
          if (typeof event.data === "string") {
            message = JSON.parse(event.data);
          } else {
            const arrayBuffer = await (event.data as Blob).arrayBuffer();
            message = ResponseMessage.fromBinary(new Uint8Array(arrayBuffer));
          }
          this.handleMessage(message);
        } catch (error) {
          console.error("Failed to process message:", error);
        }
      };
    });

    return this.wsPromise;
  }

  private handleMessage(message: ResponseMessage) {
    switch (message.payloadType) {
      case ResponseMessageType.StartGame:
        const startGameData = StartGameResponse.fromBinary(message.payload);
        console.log(startGameData);
        break;
      case ResponseMessageType.ClientState:
        const clientStateData = ClientState.fromBinary(message.payload, {
          readUnknownField: false,
        });
        this.emitter?.emit(
          ResponseMessageType.ClientState.toString(),
          clientStateData
        );
        break;
      default:
        console.warn("Unknown message type:", message.payloadType);
    }
  }

  public addEventListener(event: string, listener: (...args: any[]) => void) {
    this.emitter?.on(event, listener);
  }

  public removeEventListener(
    event: string,
    listener: (...args: any[]) => void
  ) {
    this.emitter?.off(event, listener);
  }
}
