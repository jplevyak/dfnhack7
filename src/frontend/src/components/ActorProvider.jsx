import React, { useState, useEffect } from "react";
import { createActor, _SERVICE, canisterId } from "../../../declarations/ic";

const actorContext = React.createContext(null);

export const ActorProvider = ({ children }) => {
  const [actor, setActor] = useState(null);

  useEffect(() => {
    setActor(createActor(canisterId));
  }, []);

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
    throw new Error("useActor must be used within a ActorProvider");
  }
  return context;
}
