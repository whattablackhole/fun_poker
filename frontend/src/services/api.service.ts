import { LobbyList } from "../types/lobby";
import {
  CreateLobbyRequest,
  JoinLobbyRequest,
  SpawnBotRequest,
  StartGameRequest,
} from "../types/requests";
import { User } from "../types/user";

const apiUrl = import.meta.env.VITE_API_URL;

export class ResponseError extends Error {
  statusCode?: number;

  constructor(message: string, statusCode?: number) {
    super(message);
    this.name = 'ResponseError';
    this.statusCode = statusCode;
  }
}

class ApiService {
  public static getLobbies(): Promise<LobbyList> {
    return fetch(`${apiUrl}/getLobbies`, { method: "GET" }).then((response) => {
      return (
        response.body
          ?.getReader()
          .read()
          .then((s) => {
            if (s.value) {
              let result = LobbyList.fromBinary(new Uint8Array(s.value.buffer));
              return result;
            }
            return { list: [] } as LobbyList;
          }) ?? ({ list: [] } as LobbyList)
      );
    });
  }

  public static joinLobby(request: JoinLobbyRequest) {
    return fetch(`${apiUrl}/joinLobby`, {
      method: "POST",
      body: JoinLobbyRequest.toBinary(request),
    }).then((response) => {
      return response.body?.getReader().read().then();
    });
  }

  public static startGame(request: StartGameRequest) {
    fetch(`${apiUrl}/startGame`, {
      method: "POST",
      body: StartGameRequest.toBinary(request),
    }).then();
  }

  public static createLobby(request: CreateLobbyRequest) {
    fetch(`${apiUrl}/createLobby`, {
      method: "POST",
      body: CreateLobbyRequest.toBinary(request),
    }).then();
  }

  public static spawnBot(request: SpawnBotRequest) {
    fetch(`${apiUrl}/spawnAIBot`, {
      method: "POST",
      body: SpawnBotRequest.toBinary(request),
    }).then();
  }

  public static signInByGoogle(token: string): Promise<User | undefined> {
    return fetch(`${apiUrl}/auth/signin-google`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({ Credential: token }),
    }).then((response) => {
      if (response.ok) {
        return response.json().then((data: { user: User }) => {
          return data.user;
        });
      } else {
        return undefined;
      }
    });
  }

  public static logout(): Promise<void> {
    return fetch(`${apiUrl}/auth/logout`, { credentials: "include" }).then();
  }

  public static fetchUser(): Promise<User> {
    return fetch(`${apiUrl}/auth/get-user`, { credentials: "include" })
      .then((response) => {
        if (!response.ok) {
          throw new ResponseError(response.statusText, response.status);
        }
        return response.json();
      })
      .then((data) => {
        return data.user as User;
      })
      .catch((error) => {
        throw new ResponseError(error.message);
      });
  }

  public static refreshToken(): Promise<User> {
    return fetch(`${apiUrl}/auth/refresh-token`, { credentials: "include" })
      .then((response) => {
        if (!response.ok) {
          throw new ResponseError(response.statusText, response.status);
        }
        return response.json();
      })
      .then((data) => {
        return data.user as User;
      })
      .catch((error) => {
        throw new ResponseError(error.message);
      });
  }

  public static fetchTempAccessToken(userName: string, countryCode: string): Promise<Response> {
    return fetch(`${apiUrl}/auth/unauthorized_session_token`, {
      credentials: "include",
      headers: [["Content-Type", "application/json"]],
      body: JSON.stringify({ UserName: userName, CountryCode: countryCode }),
      method: "POST",
    })
  }
}

export default ApiService;
