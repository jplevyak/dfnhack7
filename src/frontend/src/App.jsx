import React, { useEffect, useState, useRef } from "react";
import { createActor, _SERVICE, canisterId } from "../../declarations/ic";
import { AuthClient } from "@dfinity/auth-client";
import { Search } from "./components/Search";
import { Upload } from "./components/Upload";
import { useActor } from "./components/ActorProvider";

const App = () => {
  const [loggedIn, setLoggedIn] = useState(null);
  const authClientRef = useRef(null);
  const { actor, setActor } = useActor();

  useEffect(() => {
    (async () => {
      const authClient = await AuthClient.create();
      authClientRef.current = authClient;
      if (await authClient.isAuthenticated()) {
        handleAuth();
      }
    })();
  }, []);

  const onLogin = async () => {
    await authClientRef.current.login({
      identityProvider:
        process.env.DFX_NETWORK === "ic"
          ? "https://identity.ic0.app/#authorize"
          : process.env.II_LOCAL_URL,
      onSuccess: async (x) => {
        handleAuth();
      },
    });
  };

  function handleAuth() {
    setLoggedIn(true);
    if (!actor) {
      setActor(
        createActor(canisterId, {
          agentOptions: {
            identity: authClientRef.current.getIdentity(),
          },
        })
      );
    }
  }

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
