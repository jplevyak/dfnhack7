import React, { useEffect, useState, useRef } from "react";
import { createActor, _SERVICE, canisterId } from "../../declarations/ic";
import { AuthClient } from "@dfinity/auth-client";
import { Search } from "./components/Search";
import { Upload } from "./components/Upload";
import { useActor } from "./components/ActorProvider";
import "bulma/css/bulma.min.css";
import {
  Button,
  Heading,
  Container,
  Navbar,
  Footer,
  Block,
} from "react-bulma-components";
import { LoginNotice } from "./components/LoginNotice";
// import Logo from "../assets/logo.png";

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
    setActor(
      createActor(canisterId, {
        agentOptions: {
          identity: authClientRef.current.getIdentity(),
        },
      })
    );
  }

  return (
    <>
      <Navbar py="4">
        <Container>
          <Navbar.Brand
            ml="0"
            style={{
              fontWeight: "bold",
              alignItems: "center",
            }}
          >
            <img src={"/assets/logo.svg"} width="40" height="40" /> IC Notary
          </Navbar.Brand>
          <Navbar.Menu mr="0">
            <Navbar.Container></Navbar.Container>
            {loggedIn && (
              <Navbar.Container align="end">
                <Button
                  onClick={() => {
                    authClientRef.current.logout();
                    setLoggedIn(false);
                  }}
                >
                  Log out
                </Button>
              </Navbar.Container>
            )}
          </Navbar.Menu>
        </Container>
      </Navbar>
      <Container mb="6">
        <Heading textAlign="center">
          Notarized Data on the Internet Computer
        </Heading>
        {!loggedIn && <LoginNotice onLogin={onLogin} />}
        <Search
          principalId={
            loggedIn
              ? authClientRef.current.getIdentity().getPrincipal().toString()
              : null
          }
        ></Search>

        {loggedIn && (
          <Upload
            principal={authClientRef.current
              .getIdentity()
              .getPrincipal()
              .toString()}
          ></Upload>
        )}
      </Container>
      <Block textAlign="center">By team 7</Block>
    </>
  );
};

export default App;
