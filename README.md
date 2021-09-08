# DFINITY hackathon Fall 2021 Team 7

## Local install

### Install Internet Identity locally

1. run `dfx start --background` in the project folder
1. in a separate directory clone https://github.com/dfinity/internet-identity
1. cd into the newly cloned internet-identity folder
1. run `npm install`
1. run `II_ENV=development dfx deploy --no-wallet --argument '(null)'`
1. note the `internet_identity` canister ID (eg. rkp4c-7iaaa-aaaaa-aaaca-cai)

### Install the hackathon project

1. switch back to the hackathon project directory
1. create a file called `local_canisters.json` and add content
   ```
   {
    "II_LOCAL_UI_CANISTER_ID": "<id from above>"
   }
   ```
1. run `npm install`
1. run `dfx deploy`
1. run `npm start`
1. open the local frontend at http://localhost:3000/
