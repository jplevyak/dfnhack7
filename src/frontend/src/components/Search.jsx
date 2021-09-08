import React, { useState } from "react";

export const Search = ({ onSubmit }) => {
  const [term, setTerm] = useState("");

  return (
    <div>
      <input
        placeholder="Search Terms..."
        value={term}
        onChange={(e) => setTerm(e.target.value)}
      ></input>
      <button onClick={() => onSubmit}>Search</button>
    </div>
  );
};
