const { appendFile } = require("node:fs/promises");
const path = require("node:path");
const { homedir, EOL } = require("node:os");
const { randomInt } = require("node:crypto");
const { keyStores, connect, KeyPair, utils } = require("near-api-js");
const { deriveKey } = require("./derive-mpc-key");

const CREDENTIALS_DIR = ".near-credentials";
const credentialsPath = path.join(homedir(), CREDENTIALS_DIR);
const keyStore = new keyStores.UnencryptedFileSystemKeyStore(credentialsPath);

const config = {
  keyStore,
  networkId: "testnet",
  nodeUrl: "https://rpc.testnet.near.org",
};

const contractId = process.env.CONTRACT_ID;
const adminAccountId = process.env.ADMIN_ACCOUNT_ID;

async function main() {
  if (process.env.CONTROLLABLE_ACCOUNT_ID) {
    throw new Error(
      `The controllable account already exists. To override please remove "CONTROLLABLE_ACCOUNT_ID" from .env file`
    );
  }

  if (!contractId) {
    throw new Error(`"CONTRACT_ID" is missing in .env file`);
  }

  if (!adminAccountId) {
    throw new Error(`"ADMIN_ACCOUNT_ID" is missing in .env file`);
  }

  const near = await connect(config);

  const adminAccount = await near.account(adminAccountId);

  // create controllable account
  const accountId = `${Date.now()}-${randomInt(1024)}.testnet`;

  const account = await near.account(accountId);

  const keyPair = KeyPair.fromRandom("ed25519");
  await keyStore.setKey(config.networkId, accountId, keyPair);

  await adminAccount.functionCall({
    contractId: "testnet",
    methodName: "create_account",
    args: {
      new_account_id: accountId,
      new_public_key: keyPair.getPublicKey().toString(),
    },
    gas: "300000000000000",
    attachedDeposit: utils.format.parseNearAmount("0.5"), // transfer to the account 0.5 Near
  });

  console.log(`Created new account ${accountId} for interacting on its behalf`);

  // Derive MPC public key
  const derivationPath = `${adminAccountId}-${accountId}`;
  const derivedPublicKey = await deriveKey(contractId, derivationPath);

  await account.addKey(derivedPublicKey);

  console.log(
    `Added derived public key ${derivedPublicKey} to the account ${accountId}`
  );

  // Add controllable account id to .env file
  const envFilePath = path.join(__dirname, ".env");
  await appendFile(envFilePath, EOL + `CONTROLLABLE_ACCOUNT_ID=${accountId}`);
}

main().catch(console.error);
