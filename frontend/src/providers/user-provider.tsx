import { createContext, useContext } from "react";
import { User } from "../types/user";

export const UserContext = createContext<{ user: User | undefined }>({
  user: undefined,
});

export const useUser = () => {
  const context = useContext(UserContext);
  if (!context) {
    throw new Error("useUser must be used within a UserProvider");
  }
  return context;
};
