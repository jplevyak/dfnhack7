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

From this directory, run `./run_local.sh` and point your browser to http://localhost:3000
