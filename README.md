# Controlling Near account from Smart Contract

This is a simple example showing how to use chain signatues to allow your account to be controlled by a smart contract. This example is a simple smart contract with a single function that signs a transfer 0.1 Near transaction on behalf of another Near account

This example provides scripts to help you create an account on whose behalf you're acting and send Near tokens from its account to yours

## Running the project

Enter the scripts directory and install the dependencies:

```bash
cd scripts
yarn install
```

The contract is already deployed for you at `broken-rock.testnet`. To interact with it, you need to fill in your account that will be used for signing transactions.

Before proceeding to the setup, please copy `cp .env.example .env` file with environment variables and replace `ADMIN_ACCOUNT_ID` with your actual account ID

## Setup

To create an account and add public key that is managed by MPC service to be able sign on its behalf, run the following command:

```bash
yarn setup
```

## Transfer funds

To create a transaction for sending 0.1 NEAR and sign it on behalf of the account created in previous step, run the following command:

```bash
yarn transfer
```
