import React, { useState } from "react";

export const Upload = ({ principalId, onUpload, uploading, error }) => {
  let [file, setFile] = useState(null);
  function onFilesChanged(e) {
    console.log(e);
    setFile(e.target.files[0]);
  }

  async function upload(e) {
    e.preventDefault();

    onUpload(file);
  }

  return (
    <div>
      <form onSubmit={upload}>
        <input
          type="file"
          onChange={onFilesChanged}
          disabled={uploading}
          required
        ></input>
        <button>Upload & Notarize</button>
      </form>

      {error && <p>Error: {error}</p>}
      {uploading && <p>Uploading...</p>}
    </div>
  );
};
