# DFINITY hackathon Fall 2021 Team 7: Notarization Service

This is a timestamped notarization service that allows users to prove that
they held a document (or an arbitrary file) at a certain point in time. The user
can upload the file to our service, and also search and download previously uploaded 
files.

## Running locally

Prerequisites:

- The IC SDK (in particular, `dfx` should be working)
- npm >= 7
- git
- cargo 1.54
- rustc

From this directory, run `./run_local.sh`

## Usage:

The `run_local.sh` script will open a browser to the locally running Notary web application at [localhost:3000](http://localhost:3000)

### Create and Login to Internet Identity

1. Click on "Log in with Internet Identity"
2. Click on "Create an Internet Identity Anchor."
3. Enter a device name. Click on "Create"
4. At the webauthn pop-up, select "This device" or "USB security key" as you have available.
5. Click on "Confirm" on the Confirm new device page.
6. Click on "Continue" on the Congratulations page.
7. Click on "Add a recovery mechanism to an Identity Anchor"
8. Click on 'Add recovery later"
9. Click on "Proceed" on the Authorize Authentication page.

### Notorize a file

You can select from the list:

- By file upload
 - Upload a file from your browser.
- By canister fetch
 - WIP
- By content hash
 - WIP

## Problems?

If you get an error about a missing `node` package and a missing target for `wasm32-unknown-unknown`, you may need to run this:

    rustup target add wasm32-unknown-unknown

