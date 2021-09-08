import { Actor, HttpAgent } from "@dfinity/agent";
import {
  idlFactory as dfnhack7_idl,
  canisterId as ic_id,
} from "../../declarations/ic";
import { sha256 } from 'js-sha256';
const agent = new HttpAgent();
const dfnhack7 = Actor.createActor(dfnhack7_idl, { agent, canisterId: ic_id });
const params = new URLSearchParams(window.location.search);
const uploaded_sha = params.get('uploaded_sha');
const size_limit = 1000000; // no-chunking yet so be conservative for now.

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
let doc = document.getElementById("doc");
document.getElementById("upload").addEventListener('click', () => {
  if (doc.files.length > 0) {
    if (doc.files[0].size <= 1000000) {
      const reader = new FileReader();
      reader.addEventListener('load', () => {
        const sha = sha256(reader.result);
        // get principal from auth or enter separately.
        dfnhack7.notarize(null, reader.result);
        params.set('upload_sha', sha);
        window.location.href = `?${params}`;
      });
    } else {
      // print some error message
    }
    reader.readAsBinaryString(doc.files[0]);
  }
});

if (uploaded_sha) {
  // print an "updated: <uploaded sha> message
}
