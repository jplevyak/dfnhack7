import React, { useState } from "react";
import { useActor } from "./ActorProvider";
import { SearchResult, SearchResults } from "./SearchResults";

export const Search = ({ onSubmit }) => {
  const [term, setTerm] = useState("");
  const [results, setResults] = useState(null);
  const { actor } = useActor();

  const search = async (term) => {
    let results = await actor.search(term);
    setResults(results);
  };

  return (
    <div>
      Description keywords or sha256 hash: <input
        placeholder="Search Terms..."
        value={term}
        onChange={(e) => setTerm(e.target.value)}
      ></input>
      <button onClick={() => search(term)}>Search</button>

      {results && results.length > 0 && (
        <SearchResults>
          {results.map((result, index) => (
            <SearchResult result={result} key={index}></SearchResult>
          ))}
        </SearchResults>
      )}
    </div>
  );
};
