import React, { useState, useRef } from "react";
import { useActor } from "./ActorProvider";
import {
  Box,
  Form,
  Button,
  Notification,
  Block,
  Heading,
  Menu,
  Columns,
} from "react-bulma-components";

export const Upload = ({ principal }) => {
  const [file, setFile] = useState(null);
  const [note, setNote] = useState("");
  const [hash, setHash] = useState("");
  const [isPrivate, setIsPrivate] = useState(false);
  const [uploading, setUploading] = useState(false);
  const [error, setError] = useState(false);
  const [success, setSuccess] = useState(false);
  const { actor } = useActor();
  const inputFileRef = useRef();
  const [uploadType, setUploadType] = useState("file");

  function onFilesChanged(e) {
    setFile(e.target.files[0]);
  }

  async function upload(e) {
    e.preventDefault();
    setUploading(true);
    setSuccess(false);
    setError(false);
    try {
      switch (uploadType) {
        case "file":
          {
            const result = await actor.notarize(
              {
                content: Array.from(new Uint8Array(await file.arrayBuffer())),
                content_type: file.type,
              },
              note,
              isPrivate
            );

            if (result.length === 0) {
              setError("This file was already notarized");
            } else {
              setSuccess(true);
            }

            inputFileRef.current.value = null;
            setNote("");
            setIsPrivate(false);
          }
          break;
        case "hash":
          {
            const result = await actor.notarize_hash(hash, note);

            if (result.length === 0) {
              setError("This hash was already added");
            } else {
              setSuccess(true);
            }

            setHash("");
            setNote("");
            setIsPrivate(false);
          }
          break;
      }
    } catch {
      setSuccess(false);
      setError("An error occurred.");
    }
    setUploading(false);
  }

  return (
    <Box style={{ width: 700, margin: "auto" }} mt="6">
      <Heading size="4">Notarize a file</Heading>
      <Block p="1" textColor="gray" style={{ fontSize: 12 }}>
        Your principal ID: {principal}
      </Block>
      <Columns mt="4">
        <Columns.Column size="4">
          <Menu>
            <Menu.List>
              <Menu.List.Item
                active={uploadType === "file"}
                onClick={() => !uploading && setUploadType("file")}
              >
                By file upload
              </Menu.List.Item>
              <Menu.List.Item
                active={uploadType === "hash"}
                onClick={() => !uploading && setUploadType("hash")}
              >
                By content hash
              </Menu.List.Item>
              <Menu.List.Item
                active={uploadType === "http"}
                onClick={() => !uploading && setUploadType("http")}
              >
                By canister fetch
              </Menu.List.Item>
            </Menu.List>
          </Menu>
        </Columns.Column>

        {uploadType === "http" ? (
          <Columns.Column>
            <Heading size="4">todo!();</Heading>
            <Block>
              Our team initially considered a solution where the IC would be
              able to connect to external hosts via TLS and fetch assets using
              HTTP.
              <br></br>
              <br></br>
              The solution requires an external application to handle the TCP
              connection, but it is the canister that does the actual TLS
              handshake and it controls the key material, it doesn't have to
              trust the external service, since that cannot decrypt the traffic.
              <br></br>
              <br></br>
              However the idea was scrapped due to the assumption that malicious
              node operators can read the key material and use it in their own
              external client, allowing them to decrypt the message and
              fabricate new ones.
            </Block>
          </Columns.Column>
        ) : (
          <Columns.Column>
            <form onSubmit={upload}>
              {uploadType === "file" && (
                <Form.Field kind="group">
                  <Form.Control>
                    <Form.InputFile
                      domRef={inputFileRef}
                      type="file"
                      onChange={onFilesChanged}
                      disabled={uploading}
                      required
                    ></Form.InputFile>
                  </Form.Control>
                  {file && (
                    <Block style={{ lineHeight: "40px" }}>{file.name}</Block>
                  )}
                </Form.Field>
              )}
              {uploadType === "hash" && (
                <Form.Field kind="group">
                  <Form.Control fullwidth>
                    <Form.Input
                      placeholder="Hash of document..."
                      value={hash}
                      onChange={(e) => setHash(e.target.value)}
                      disabled={uploading}
                      required
                    ></Form.Input>
                  </Form.Control>
                  {file && (
                    <Block style={{ lineHeight: "40px" }}>{file.name}</Block>
                  )}
                </Form.Field>
              )}

              <Form.Field>
                <Form.Control>
                  <Form.Textarea
                    value={note}
                    disabled={uploading}
                    onChange={(e) => setNote(e.target.value)}
                    required
                    placeholder="Description..."
                  ></Form.Textarea>
                </Form.Control>
              </Form.Field>
              <Form.Field>
                <Form.Control>
                  <Form.Checkbox
                    onChange={(e) => setIsPrivate(e.target.checked)}
                    checked={isPrivate}
                    disabled={uploading}
                  >
                    Private file (only the owner can download it)
                  </Form.Checkbox>
                </Form.Control>
              </Form.Field>
              <Form.Field>
                <Form.Control>
                  <Button disabled={uploading} color="primary" outlined={true}>
                    {uploading ? "Uploading..." : "Upload & Notarize"}
                  </Button>
                </Form.Control>
              </Form.Field>
            </form>
            {error && (
              <Notification mt="4" color="danger">
                Error: {error}
              </Notification>
            )}
            {success && (
              <Notification mt="4" color="success">
                Upload successful
              </Notification>
            )}
          </Columns.Column>
        )}
      </Columns>
    </Box>
  );
};
