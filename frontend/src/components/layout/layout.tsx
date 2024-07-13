import { Outlet, useLocation } from "react-router-dom";
import CreateLobbyDialog from "../popups/create-lobby-dialog";
import NavigationHeader from "../navigation_header/navigation-header";
import { Container } from "@mui/material";
import GoogleSignIn from "../googel-signin/google-signin";
import { useUserContext } from "../../contexts/user-context";

export default function Layout({ login, logout }: any) {
  const location = useLocation();
  const { user } = useUserContext();

  return (
    <div>
      <NavigationHeader>
        <Container
          sx={{
            flexDirection: "row",
            display: "flex",
            justifyContent: "space-between",
          }}
        >
          <CreateLobbyDialog />

          <div style={{ display: "flex", alignItems: "flex-end" }}>
            {user ? (
              <div
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: "15px",
                }}
              >
                <div>{user.name}</div>
                <button onClick={logout}>Logout</button>
              </div>
            ) : (
              <GoogleSignIn signInHandler={login} />
            )}
          </div>
        </Container>
      </NavigationHeader>
      <Outlet />
    </div>
  );
}
