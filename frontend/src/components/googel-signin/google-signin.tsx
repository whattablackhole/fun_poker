import { useEffect } from "react";
import { User } from "../../types/user";
import AuthService from "../../services/auth.service";

function GoogleSignIn({
  signInHandler,
}: {
  signInHandler: (user: User) => void;
}) {
  useEffect(() => {
    const handleCredentialResponse = async (response: any) => {
      let data = await AuthService.signinByGoogle(response.credential);
      
      if (data?.user) {
        await signInHandler(data.user);
      } else {
        console.log("show notification about signin error");
      }
    };
    
    const initializeGoogleSignIn = () => {
      if (window.google) {
        window.google.accounts.id.initialize({
          client_id:
            "355255212720-fcovem0bl4uo6au8qpcmc6f6kjbs6mhv.apps.googleusercontent.com",
          callback: handleCredentialResponse,
        });
        window.google.accounts.id.renderButton(
          document.getElementById("google-signin-button")!,
          { theme: "outline", size: "large" }
        );
      }
    };

    initializeGoogleSignIn();
  }, []);

  return <div id="google-signin-button"></div>;
}

export default GoogleSignIn;
