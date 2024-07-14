import EventEmitter from "eventemitter3";
import {
  ResponseMessage,
  ResponseMessageType,
  StartGameResponse,
} from "../types/responses";
import { ClientState } from "../types/client_state";

export default class WebSocketService {
  private ws: WebSocket | null = null;
  private emitter: EventEmitter | null = null;
  private wsPromise: Promise<void> | null = null;
  private url: string | null = null;

  public reconnect(): Promise<void> {
    if (!this.url) {
      throw new Error("Websocket is not connected!");
    }

    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.ws.close();
    }

    if (this.wsPromise) {
      return this.wsPromise;
    }

    return this.connect(this.url);
  }

  public async disconnect() {
    if (this.ws) {
      this.ws.onclose = null;
      this.ws.close();
      this.ws = null;
    }

    if (this.emitter) {
      this.emitter.emit("Disconnecting");
      this.emitter.removeAllListeners();
      this.emitter = null;
    }

    this.wsPromise = null;
  }

  public sendMessage(message: Uint8Array) {
    if (this.ws) {
      this.ws.send(message);
    } else {
      throw Error("Websocket is not initialized");
    }
  }

  public connect(url: string): Promise<void> {
    if (this.wsPromise) {
      throw Error(
        "An attempt to establish a new connection while connection is being processed"
      );
    }

    this.wsPromise = new Promise((resolve) => {
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
