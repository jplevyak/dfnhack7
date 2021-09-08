import React from "react";
import { canisterId } from "../../../declarations/ic";

export const SearchResult = ({ result }) => {
  let url ="https://" + canisterId + ".ic0.app"; 
  if (process.env.NODE_ENV !== "production") {
    url = "http://" + canisterId + ".localhost:8000"
  }
  url += "/" + result.hash;
  return (
    <li> <a href={url}>{result.hash}</a> by {result.owner.toString()} on {(new Date(Number(result.created / 1000n))).toString()} : {result.description}
    </li>
  );
};

export const SearchResults = ({ children }) => {
  return <ul>{children}</ul>;
};
