import React from "react";

export const SearchResult = ({ result }) => {
  return (
    <li>
      {result.hash} : {result.description}
    </li>
  );
};

export const SearchResults = ({ children }) => {
  return <ul>{children}</ul>;
};
