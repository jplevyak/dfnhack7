#!/usr/bin/env bash

dfx start --background
git submodule add https://github.com/dfinity/internet-identity
cd internet-identity
npm install
II_ENV=development dfx deploy --no-wallet --argument '(null)'
II_CANISTER_ID=$(dfx canister --no-wallet id internet_identity)
cd ..
# TODO: echo the identity into local_canisters.json
cat <<EOF >> local_canisters.json
   {
    "II_LOCAL_UI_CANISTER_ID": "$II_CANISTER_ID"
   }
EOF
npm install
dfx deploy
npm start

unameOut="$(uname -s)"
case "${unameOut}" in
    Linux*)     opener=xdg-open;;
    Darwin*)    opener=open;;
esac

$opener http://localhost:3000
