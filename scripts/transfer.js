const { connect, keyStores, providers } = require("near-api-js");
const { homedir } = require("node:os");
const path = require("node:path");
const { deriveKey } = require("./derive-mpc-key");

const CREDENTIALS_DIR = ".near-credentials";
const credentialsPath = path.join(homedir(), CREDENTIALS_DIR);
const keyStore = new keyStores.UnencryptedFileSystemKeyStore(credentialsPath);

const сonfig = {
  networkId: "testnet",
  keyStore: keyStore,
  nodeUrl: "https://rpc.testnet.near.org",
  walletUrl: "https://testnet.mynearwallet.com/",
  helperUrl: "https://helper.testnet.near.org",
  explorerUrl: "https://testnet.nearblocks.io",
};

const contractId = process.env.CONTRACT_ID;
const adminAccountId = process.env.ADMIN_ACCOUNT_ID;
const controllableAccountId = process.env.CONTROLLABLE_ACCOUNT_ID;

async function main() {
  if (!controllableAccountId) {
    throw new Error(
      `You must run "setup" script first to create a controllable account!`
    );
  }

  if (!contractId) {
    throw new Error(`"CONTRACT_ID" is missing in .env file`);
  }

  if (!adminAccountId) {
    throw new Error(`"ADMIN_ACCOUNT_ID" is missing in .env file`);
  }

  const near = await connect(сonfig);

  const adminAccount = await near.account(adminAccountId);

  // Derive MPC public key
  const derivationPath = `${adminAccountId}-${controllableAccountId}`;
  const derivedPublicKey = await deriveKey(contractId, derivationPath);

  // Get the nonce of the key
  const accessKey = await near.connection.provider.query({
    request_type: "view_access_key",
    account_id: controllableAccountId,
    public_key: derivedPublicKey,
    finality: "optimistic",
  });
  const nonce = accessKey.nonce;

  // Get recent block hash
  const block = await near.connection.provider.block({
    finality: "final",
  });
  const blockHash = block.header.hash;

  // Prepare transaction arguments
  const transaction_args = {
    signer_id: controllableAccountId,
    signer_pk: derivedPublicKey,
    nonce: (nonce + 1).toString(),
    block_hash: blockHash,
  };

  // Call the contract to charge the subscription
  const outcome = await adminAccount.functionCall({
    contractId: contractId,
    methodName: "transfer_on_behalf_of",
    args: {
      args: transaction_args,
    },
    gas: "300000000000000",
    attachedDeposit: "100000000000000000000000", // 0.1 NEAR is enough in most cases to pay MPC fee
  });

  // Get the signed transaction from the outcome
  result = providers.getTransactionLastResult(outcome);
  const signedTx = new Uint8Array(result);

  // Send the signed transaction
  const transfer_outcome = await near.connection.provider.sendJsonRpc(
    "broadcast_tx_commit",
    [Buffer.from(signedTx).toString("base64")]
  );

  console.log(
    `${сonfig.explorerUrl}/txns/${transfer_outcome.transaction_outcome.id}`
  );
}

main().catch(console.error);
