import * as React from "react";
import * as ReactDOM from "react-dom";
import App from "./App";
import { ActorProvider } from "./components/ActorProvider";

ReactDOM.render(
  <ActorProvider>
    <App />
  </ActorProvider>,
  document.getElementById("root")
);
