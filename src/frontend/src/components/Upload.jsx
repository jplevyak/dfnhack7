import React, { useState, useRef } from "react";
import { useActor } from "./ActorProvider";

export const Upload = ({}) => {
  const [file, setFile] = useState(null);
  const [note, setNote] = useState("");
  const [uploading, setUploading] = useState(false);
  const [error, setError] = useState(false);
  const [success, setSuccess] = useState(false);
  const { actor } = useActor();
  const inputRef = useRef();

  function onFilesChanged(e) {
    setFile(e.target.files[0]);
  }

  async function upload(e) {
    e.preventDefault();
    setUploading(true);
    setSuccess(false);
    setError(false);
    try {
      const result = await actor.notarize(
        {
          content: Array.from(new Uint8Array(await file.arrayBuffer())),
          content_type: file.type,
        },
        note
      );

      if (result.length === 0) {
        setError("This file was already notarized");
      } else {
        setSuccess(true);
      }

      inputRef.current.value = null;
    } catch {
      setSuccess(false);
      setError("An error occurred.");
    }
    setUploading(false);
  }

  return (
    <div>
      <form onSubmit={upload}>
        <input
          ref={inputRef}
          type="file"
          onChange={onFilesChanged}
          disabled={uploading}
          required
        ></input>
        <button disabled={uploading}>
          {uploading ? "Uploading..." : "Upload & Notarize"}
        </button>
        <textarea
          value={note}
          onChange={(e) => setNote(e.target.value)}
          required
          placeholder="Description..."
        ></textarea>
      </form>

      {error && <p>Error: {error}</p>}
      {uploading && <p>Uploading...</p>}
      {success && <p>Upload successful</p>}
    </div>
  );
};
