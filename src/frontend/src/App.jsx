import React, { useEffect, useState, useRef } from "react";
import { ic } from "../../declarations/ic";
import { AuthClient } from "@dfinity/auth-client";
import { Search } from "./components/Search";
import { Upload } from "./components/Upload";

const App = () => {
  const [loggedIn, setLoggedIn] = useState(null);
  const authClientRef = useRef(null);

  useEffect(() => {
    (async () => {
      const authClient = await AuthClient.create();
      authClientRef.current = authClient;
      if (await authClient.isAuthenticated()) {
        setLoggedIn(true);
      }
    })();
  });

  const onLogin = async () => {
    await authClientRef.current.login({
      identityProvider:
        process.env.DFX_NETWORK === "ic"
          ? "https://identity.ic0.app/#authorize"
          : process.env.II_LOCAL_URL,
      onSuccess: async (x) => {
        setLoggedIn(true);
      },
    });
  };

  return (
    <div>
      <h1>Notarized Data on the Internet Computer</h1>
      {loggedIn && (
        <p>
          Your principal:{" "}
          {authClientRef.current.getIdentity().getPrincipal().toString()}
        </p>
      )}
      <Search></Search>
      {loggedIn ? (
        <>
          <Upload></Upload>
          <button
            onClick={() => {
              authClientRef.current.logout();
              setLoggedIn(false);
            }}
          >
            Log out
          </button>
        </>
      ) : (
        <>
          <p>To notarize documents log in with II</p>
          <button onClick={onLogin}>Log in with Internet Identity</button>
        </>
      )}
    </div>
  );
};

export default App;
