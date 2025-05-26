import { BigNumber } from 'bignumber.js';

import * as anchor from '@coral-xyz/anchor';
import {
  createAssociatedTokenAccount,
  createMint,
  mintTo,
} from '@solana/spl-token';

export const TEN_BIGNUM = BigNumber(10);

export function calculateTransactionSize(
	ixs: anchor.web3.TransactionInstruction[],
	recentBlockhash: string,
) {
	const size = new anchor.web3.TransactionMessage({
		instructions: ixs,
		payerKey: ixs[0].keys[0].pubkey,
		recentBlockhash,
	})
		.compileToV0Message()
		.serialize().length;

	// pub struct Proposal {
	// 	pub vault: Pubkey,
	// 	pub proposal_stage: ProposalStage,
	// 	pub created_date: i64,
	// 	pub expiry_date: i64,
	// 	pub is_executed: bool,
	// 	pub name: String,
	// 	pub actions: Vec<Action>,
	// }

	const withExtraSpace = 32 + 1 + 8 + 8 + 1 + 64 + size + /** extra */ 20;

	return withExtraSpace;
}

type ProposalStage =
	| {
			draft: {};
	  }
	| { completed: {} }
	| { cancelled: {} }
	| { failed: {} };

type AccountSpec = { pubkey: anchor.web3.PublicKey; isSigner: boolean; isWritable: boolean };

type Action = {
	programId: anchor.web3.PublicKey;
	accountSpecs: AccountSpec[];
	data: Buffer<ArrayBufferLike>;
};

type Proposal = {
	vault: anchor.web3.PublicKey;
	name: string;
	proposalStage: ProposalStage;
	createdDate: anchor.BN;
	expiryDate: anchor.BN;
	isExecuted: boolean;
	actions: Action[];
};

export function printProposalAccount(proposalAccount: Proposal) {
	console.log("Proposal Account:");
	console.log(
		JSON.stringify(
			{
				...proposalAccount,
				createdDate: unixTimestampToDate(proposalAccount.createdDate.toNumber()),
				expiryDate: unixTimestampToDate(proposalAccount.expiryDate.toNumber()),
				vault: proposalAccount.vault.toString(),
				actions: proposalAccount.actions.map((action) => ({
					accountSpecs: action.accountSpecs.map((spec) => ({
						...spec,
						pubkey: spec.pubkey.toString(),
					})),

					data: action.data.toString("base64"),
					programId: action.programId.toString(),
				})),
			},
			null,
			2,
		),
	);
}

export function unixTimestampToDate(unixTimestamp: number) {
	return new Date(unixTimestamp * 1000);
}

type Vault = {
	owner: anchor.web3.PublicKey;
	createdDate: anchor.BN;
	signerBump: number;
};

export function printVaultAccount(vaultAccount: Vault) {
	console.log("Vault Account:"),
		console.log(
			JSON.stringify(
				{
					owner: vaultAccount.owner.toString(),
					signerBump: vaultAccount.signerBump,
					createdAt: unixTimestampToDate(vaultAccount.createdDate.toNumber()),
				},
				null,
				2,
			),
		);
}

export function sleep(ms: number) {
	return new Promise((r) => setTimeout(r, ms));
}

export async function getBlockTime(connection: anchor.web3.Connection) {
	const time = await connection.getBlockTime(await connection.getSlot());
	return time!;
}

export async function createNewMint(
	connection: anchor.web3.Connection,
	payer: anchor.web3.Keypair,
	mintKeypair: anchor.web3.Keypair,
	mintDecimals: number,
	destination?: anchor.web3.PublicKey,
) {
	// const mintKeypair = anchor.web3.Keypair.generate();
	const mint = await createMint(
		connection,
		payer,
		payer.publicKey,
		payer.publicKey,
		mintDecimals,
		mintKeypair,
	);
	// const mint = mintKeypair.publicKey;
	console.log("\tMint:", mint.toString());

	const destinationTokenAccount = await createAssociatedTokenAccount(
		connection,
		payer,
		mint,
		destination ? destination : payer.publicKey,
	);
	// console.log("\tTokenAccount:", destinationTokenAccount.toString());

	const mintToSignature = await mintTo(
		connection,
		payer,
		mint,
		destinationTokenAccount,
		payer,
		BigInt(1000000000) * BigInt(10 ** mintDecimals),
	);
	console.log("\tMintToSignature:", mintToSignature);

	return mint;
}
