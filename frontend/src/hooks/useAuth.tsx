import { useEffect, useState } from "react";
import { User } from "../types/user";
import AuthService from "../services/auth.service";

export const useAuth = () => {
  const [user, setUser] = useState<User | undefined>(undefined);

  useEffect(() => {
    AuthService.trySilentAuthentication().then((user) => {
      setUser(user);
    });
  }, []);

  const login = (user: User) => {
    setUser(user);
  };

  const logout = async () => {
    setUser(undefined);
    await AuthService.logout();
  };

  return { user, login, logout };
};
