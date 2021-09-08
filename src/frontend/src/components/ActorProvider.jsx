import React, { useState } from "react";

const actorContext = React.createContext(null);

export const ActorProvider = ({ children }) => {
  const [actor, setActor] = useState(null);

  return (
    <actorContext.Provider
      value={{
        actor,
        setActor,
      }}
    >
      {children}
    </actorContext.Provider>
  );
};

export function useActor() {
  const context = React.useContext(actorContext);
  if (context === undefined) {
    throw new Error("useActor must be used within a CountProvider");
  }
  return context;
}
