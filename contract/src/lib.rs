use near_sdk::json_types::U64;
use near_sdk::{borsh, env, near, AccountId, PanicOnDefault, Promise, PromiseError};
use omni_transaction::near::types::{Action, Signature, TransferAction, U128 as OmniU128};
// needed to convert string into BlockHash
use omni_transaction::near::utils::PublicKeyStrExt;
use omni_transaction::near::NearTransaction;
use omni_transaction::{TransactionBuilder, TxBuilder, NEAR};
use sha2::{Digest, Sha256};
use signer::{ext_signer, SignRequest, SignResult};

pub mod signer;

const OMNI_DEPOSIT: OmniU128 = OmniU128(100_000_000_000_000_000_000_000); // 0.1 NEAR

#[near(contract_state)]
#[derive(PanicOnDefault)]
struct Contract {
    mpc_contract_id: AccountId,
}

#[near(serializers = [json])]
struct TransactionArguments {
    pub signer_id: AccountId,
    pub signer_pk: omni_transaction::near::types::PublicKey,
    pub nonce: U64,
    pub block_hash: String,
}

#[near]
impl Contract {
    #[init]
    pub fn new(mpc_contract_id: AccountId) -> Self {
        Self { mpc_contract_id }
    }

    #[payable]
    pub fn transfer_on_behalf_of(&mut self, args: TransactionArguments) -> Promise {
        let action = Action::Transfer(TransferAction {
            deposit: OMNI_DEPOSIT,
        });

        // Add the action to the actions vector
        let actions = vec![action];

        // Build the transaction
        let near_tx = TransactionBuilder::new::<NEAR>()
            .signer_id(args.signer_id.to_string())
            .signer_public_key(args.signer_pk)
            .nonce(args.nonce.0)
            .receiver_id(env::predecessor_account_id().to_string())
            .block_hash(args.block_hash.to_block_hash().unwrap())
            .actions(actions)
            .build();

        // Create the paylaod, hash it and convert to a 32-byte array
        let serialized_near_tx = borsh::to_vec(&near_tx).unwrap();
        let hashed_payload = hash_payload(&serialized_near_tx);
        let mpc_payload: [u8; 32] = hashed_payload
            .try_into()
            .unwrap_or_else(|e| panic!("Failed to convert payload {:?}", e));

        let mpc_deposit = env::attached_deposit();
        let key_version = 0;
        let derivation_path = format!(
            "{}-{}",
            env::predecessor_account_id().to_string(),
            args.signer_id.to_string()
        );

        // Call MPC contract
        ext_signer::ext(self.mpc_contract_id.clone())
            .with_attached_deposit(mpc_deposit)
            .sign(SignRequest::new(mpc_payload, derivation_path, key_version))
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(near_sdk::Gas::from_tgas(50))
                    .with_unused_gas_weight(0)
                    .sign_callback(serialized_near_tx),
            )
    }

    #[private]
    pub fn sign_callback(
        &self,
        #[callback_result] result: Result<SignResult, PromiseError>,
        serialized_near_tx: Vec<u8>,
    ) -> Vec<u8> {
        if let Ok(sign_result) = result {
            // Transform into a secp256k1 signature
            let omni_signature = Signature::SECP256K1(sign_result.into());

            let near_tx = borsh::from_slice::<NearTransaction>(&serialized_near_tx).unwrap();
            // Add signature to transaction and return
            near_tx.build_with_signature(omni_signature)
        } else {
            let error = result.unwrap_err();
            panic!("Callback failed with error {:?}", error);
        }
    }
}

// Function to hash payload
pub fn hash_payload(payload: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(payload);
    let result = hasher.finalize();
    result.into()
}
