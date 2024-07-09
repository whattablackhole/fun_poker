import { User } from "../types/user";
import ApiService, { AuthData } from "./api.service";

export default class AuthService {
  private static accessTokenExpireTimeKey = "accessTokenExpireTime";
  private static refreshTokenExpireTimeKey = "refreshTokenExpireTime";

  private static refreshTimer: NodeJS.Timeout | null = null;

  public static async signinByGoogle(
    token: string
  ): Promise<AuthData | undefined> {
    try {
      let authData = await ApiService.signInByGoogle(token);

      localStorage.setItem(
        this.accessTokenExpireTimeKey,
        authData.accessTokenExpireTime.toString()
      );
      localStorage.setItem(
        this.refreshTokenExpireTimeKey,
        authData.refreshTokenExpireTime.toString()
      );
      this.setRefreshTimer(
        (authData.accessTokenExpireTime - Date.now() / 1000 - 60) * 1000
      );

      return authData;
    } catch (err) {
      console.error(err);
    }
  }

  public static setRefreshTimer(timeout: number) {
    if (this.refreshTimer != null) {
      clearTimeout(this.refreshTimer);
    }

    this.refreshTimer = setTimeout(() => {
      this.tryRefreshToken();
    }, timeout);
  }

  public static isRefreshTokenExpired(): boolean {
    let refreshTokenExpireTimeString = localStorage.getItem(
      this.refreshTokenExpireTimeKey
    );

    if (refreshTokenExpireTimeString === null) {
      return true;
    }

    let refreshTokenExpireTime = Number.parseInt(refreshTokenExpireTimeString);

    if (
      !Number.isNaN(refreshTokenExpireTime) &&
      Date.now() / 1000 < refreshTokenExpireTime
    ) {
      return false;
    }

    return true;
  }

  public static isAccessTokenExpired(): boolean {
    let accessTokenExpireTimeString = localStorage.getItem(
      this.accessTokenExpireTimeKey
    );

    if (accessTokenExpireTimeString === null) {
      return true;
    }

    let accessTokenExpireTime = Number.parseInt(accessTokenExpireTimeString);

    if (
      !Number.isNaN(accessTokenExpireTime) &&
      Date.now() / 1000 < accessTokenExpireTime
    ) {
      return false;
    }

    return true;
  }

  public static async logout() {
    if (this.refreshTimer) {
      clearTimeout(this.refreshTimer);
      this.refreshTimer = null;
    }

    localStorage.removeItem(this.accessTokenExpireTimeKey);
    localStorage.removeItem(this.refreshTokenExpireTimeKey);
    await ApiService.logout();
  }

  public static async trySilentAuthentication(): Promise<User | undefined> {
    if (!this.isAccessTokenExpired()) {
      try {
        let user = await ApiService.fetchUser();
        return user;
      } catch (err) {
        if (!this.isRefreshTokenExpired()) {
          return await this.tryRefreshToken();
        } else {
          console.error(err);
        }
      }
    } else if (!this.isRefreshTokenExpired()) {
      return await this.tryRefreshToken();
    }
  }

  public static async tryRefreshToken() {
    try {
      let authData = await ApiService.refreshToken();

      localStorage.setItem(
        this.accessTokenExpireTimeKey,
        authData.accessTokenExpireTime.toString()
      );
      localStorage.setItem(
        this.refreshTokenExpireTimeKey,
        authData.refreshTokenExpireTime.toString()
      );
      this.setRefreshTimer(
        (authData.accessTokenExpireTime - Date.now() / 1000 - 60) * 1000
      );

      return authData.user;
    } catch (err) {
      console.error(err);
    }
  }
}
