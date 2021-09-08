import React from "react";
import { canisterId } from "../../../declarations/ic";
import { Block, Heading, Button, Icon } from "react-bulma-components";

const FieldLabel = ({ children }) => {
  return (
    <span style={{ display: "inline-block", width: 100 }}>{children}</span>
  );
};

export const SearchResult = ({ result, makePublic }) => {
  let url = "https://" + canisterId + ".ic0.app";
  if (process.env.NODE_ENV !== "production") {
    url = "http://" + canisterId + ".localhost:8000";
  }
  url += "/" + result.hash;
  return (
    <Block
      style={{ borderBottom: "1px solid #aaa", display: "flex" }}
      mb="2"
      mt="2"
      pb="2"
      px="1"
    >
      <div style={{ flex: 1 }}>
        <Heading size="6" mb="1">
          {result.description}
        </Heading>
        <div style={{ fontSize: "12px" }}>
          <div>
            <FieldLabel>Added on: </FieldLabel>
            {new Date(Number(result.created / 1000000n)).toLocaleString()}
          </div>
          <div>
            <FieldLabel>Content: </FieldLabel>
            <a href={url} target="_blank">
              {result.hash}
            </a>
          </div>
          <div>
            <FieldLabel>Added by: </FieldLabel>
            {result.owner.toString()}
          </div>
        </div>
      </div>
      <Button style={{ border: "none" }} onClick={() => makePublic(result)}>
        <img width="24" height="24" src="/assets/private.svg"></img>
      </Button>
    </Block>
  );
};
