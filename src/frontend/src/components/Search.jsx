import React, { useState } from "react";
import { useActor } from "./ActorProvider";
import { SearchResult, SearchResults } from "./SearchResults";
import { Form, Button, Box, Columns, Block } from "react-bulma-components";

export const Search = ({ onSubmit }) => {
  const [term, setTerm] = useState("");
  const [results, setResults] = useState(null);
  const { actor } = useActor();

  const search = async (term) => {
    let results = await actor.search(term);
    setResults(results);
  };

  const makePublic = async (result) => {
    // console.log(result);
  };

  return (
    <Columns mt="6">
      <Box style={{ width: 700, margin: "auto" }}>
        <Form.Field kind="addons" size="large">
          <Form.Control fullwidth={true}>
            <Form.Input
              placeholder="Search in description, hash, principals..."
              value={term}
              onChange={(e) => { setTerm(e.target.value); search(e.target.value); }}
            />
          </Form.Control>
          <Form.Control>
            <Button onClick={() => search(term)} color="primary">
              Search
            </Button>
          </Form.Control>
        </Form.Field>

        {results && results.length > 0 && (
          <>
            <Block my="4" textWeight="bold" px="1">
              {results.length} result(s)
            </Block>
            {results.map((result, index) => (
              <SearchResult
                result={result}
                key={index}
                makePublic={makePublic}
              ></SearchResult>
            ))}
          </>
        )}
        {results && results.length === 0 && (
          <Block my="4" textAlign="center" textWeight="bold">
            No results for {term}
          </Block>
        )}
      </Box>
    </Columns>
  );
};
