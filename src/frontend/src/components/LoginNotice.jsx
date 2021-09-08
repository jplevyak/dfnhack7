import React from "react";
import { Container, Button, Box, Heading } from "react-bulma-components";

export const LoginNotice = ({ onLogin }) => {
  return (
    <Container mt="6">
      <Box
        style={{ width: 500, margin: "auto" }}
        textAlign="center"
        className="has-background-link-light"
      >
        <Heading size="4" textAlign="center">
          To notarize documents log in with II
        </Heading>
        <Button onClick={onLogin} colorVariant="dark" color="light">
          Log in with Internet Identity
        </Button>
      </Box>
    </Container>
  );
};
