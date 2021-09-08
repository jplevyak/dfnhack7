import React from "react";

export const Upload = ({ principalId, onUpload, uploading, error }) => {
  function onFilesChanged(e) {
    console.log(e);
  }

  async function upload(e) {
    e.preventDefault();

    onUpload(e);
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
