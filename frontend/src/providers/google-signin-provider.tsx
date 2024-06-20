import React, { useEffect } from 'react';

const GoogleSignIn: React.FC = () => {
  useEffect(() => {
    const handleCredentialResponse = (response: any) => {
     // Decode JWT token to get user information
    //   const userObject = jwt_decode(response.credential);
    //   console.log('ID: ' + userObject.sub);
    //   console.log('Name: ' + userObject.name);
    //   console.log('Image URL: ' + userObject.picture);
    //   console.log('Email: ' + userObject.email);
    };

    const initializeGoogleSignIn = () => {
      if (window.google) {
        window.google.accounts.id.initialize({
          client_id: '355255212720-fcovem0bl4uo6au8qpcmc6f6kjbs6mhv.apps.googleusercontent.com',
          callback: handleCredentialResponse,
        });
        window.google.accounts.id.renderButton(
          document.getElementById('google-signin-button')!,
          { theme: 'outline', size: 'large' }
        );
      }
    };

    initializeGoogleSignIn();
  }, []);

  return <div id="google-signin-button"></div>;
};

export default GoogleSignIn;