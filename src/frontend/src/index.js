import { Actor, HttpAgent } from "@dfinity/agent";
import {
  idlFactory as dfnhack7_idl,
  canisterId as ic_id,
} from "dfx-generated/dfnhack7";

const agent = new HttpAgent();
const dfnhack7 = Actor.createActor(dfnhack7_idl, { agent, canisterId: ic_id });

function escapeHTML(unsafeText) {
  let div = document.createElement("div");
  div.innerText = unsafeText;
  return div.innerHTML;
}

async function get_results() {
  const terms = document.getElementById("terms").value.toString();
  const results = await dfnhack7.search(terms);
  var t = "";
  if (results.length == 0) {
    t = "<br><strong>No matches found.</strong><br>";
  } else {
    t += "<ul>";
    for (const r of results) {
      t += "<li>";
      t += r.datum + " : " + escapeHTML(r.description);
      t += "</li>";
    }
    t += "</ul>";
  }
  document.getElementById("results").innerHTML = t;
}
document.getElementById("search").addEventListener("click", get_results);
document.getElementById("terms").onchange = get_results;
