import React from "react";
import { canisterId } from "../../../declarations/ic";
import { Block, Heading, Button, Icon } from "react-bulma-components";

const FieldLabel = ({ children }) => {
  return (
    <span style={{ display: "inline-block", width: 100 }}>{children}</span>
  );
};

export const SearchResult = ({ result }) => {
  let url = null;
  if (result.has_datum && !result.hidden) {
    url ="https://" + canisterId + ".ic0.app"; 
    if (process.env.NODE_ENV !== "production") {
      url = "http://" + canisterId + ".localhost:8000"
    }
    url += "/" + result.hash;
  }
  let result_link;
  if (url) {
    result_link = <a href={url}>{result.hash}</a>
  } else {
    result_link = result.hash + result.has_datum + result.hidden
  }
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
            {result_link}
          </div>
          <div>
            <FieldLabel>Added by: </FieldLabel>
            {result.owner.toString()}
          </div>
        </div>
      </div>
      <Button outlined={false}>
        <img
          width="24"
          height="24"
          src="data:image/svg+xml;base64,PD94bWwgdmVyc2lvbj0iMS4wIiA/PjxzdmcgZGF0YS1uYW1lPSJMYXllciAxIiBpZD0iTGF5ZXJfMSIgdmlld0JveD0iMCAwIDEwMCAxMDAiIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyI+PHRpdGxlLz48cGF0aCBkPSJNMjUuOTQsNTIuN2EyLDIsMCwwLDEsMi43Ny0uNThMNDgsNjQuNzJWMjBhMiwyLDAsMCwxLDQsMFY2NC43MmwxOS4yOS0xMi42YTIsMiwwLDAsMSwyLjE5LDMuMzVMNTEuMDksNzAuMDhsMCwwYTEuODYsMS44NiwwLDAsMS0uNDMuMmwtLjEyLDBhMS44OSwxLjg5LDAsMCwxLTEsMGwtLjEyLDBhMS44NiwxLjg2LDAsMCwxLS40My0uMmwwLDBMMjYuNTIsNTUuNDdBMiwyLDAsMCwxLDI1Ljk0LDUyLjdaTTc2LjQ4LDc4aC01M2EyLDIsMCwxLDAsMCw0aDUzYTIsMiwwLDEsMCwwLTRaIi8+PC9zdmc+"
        ></img>
      </Button>
    </Block>
  );
};
