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
    <Box style={{ width: 700, margin: "auto" }} mt="6">
      <Heading size="4">Upload a file to notarize</Heading>
      <Block p="1" textColor="gray" style={{ fontSize: 12 }}>
        Your principal ID: {principal}
      </Block>
      <Columns mt="4">
        <Columns.Column size="4">
          <Menu>
            <Menu.List>
              <Menu.List.Item active>By file upload</Menu.List.Item>
              <Menu.List.Item>By custom hash</Menu.List.Item>
              <Menu.List.Item>By canister fetch</Menu.List.Item>
            </Menu.List>
          </Menu>
        </Columns.Column>
        <Columns.Column>
          <form onSubmit={upload}>
            <Form.Field kind="group">
              <Form.Control>
                <Form.InputFile
                  domRef={inputRef}
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

            <Form.Field>
              <Form.Control>
                <Form.Textarea
                  onChange={(e) => setNote(e.target.value)}
                  required
                  placeholder="Description..."
                >
                  {note}
                </Form.Textarea>
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
      </Columns>
    </Box>
  );
};
