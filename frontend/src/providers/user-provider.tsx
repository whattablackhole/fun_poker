import { createContext, useContext } from "react";


export const UserContext = createContext<{user: { id: number } | undefined } | undefined>(undefined);

export const useUser = () => {
    const context = useContext(UserContext);
    if (!context) {
      throw new Error("useUser must be used within a UserProvider");
    }
    return context;
};