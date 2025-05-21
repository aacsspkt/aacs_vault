import { assert } from 'chai';

import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';

import { AacsVault } from '../target/types/aacs_vault';
import {
  calculateTransactionSize,
  getBlockTime,
} from './shared';

describe("Create proposal and execute proposal flow", () => {
	// Configure the client to use the local cluster.
	anchor.setProvider(anchor.AnchorProvider.env());
	const provider = anchor.getProvider();
	const program = anchor.workspace.aacsVault as Program<AacsVault>;

	const VAULT_SIGNER_PREFIX = anchor.utils.bytes.utf8.encode("aacs_vault_signer");
	const ONE_MONTH = 30 * 24 * 60 * 60;

	const payer = provider.wallet!.payer!;
	const payerPublicKey = payer.publicKey;
	const vault = anchor.web3.Keypair.generate();
	const vaultPublicKey = vault.publicKey;
	const [vaultSigner, vaultBump] = anchor.web3.PublicKey.findProgramAddressSync(
		[VAULT_SIGNER_PREFIX, vaultPublicKey.toBuffer()],
		program.programId,
	);

	const proposals = [anchor.web3.Keypair.generate()];

	describe("flow: createVault() -> createProposal() -> executeProposal()", () => {
		it("works as expected", async () => {
			// Add your test here.
			const createVaultSignature = await program.methods
				.createVault({ owner: payerPublicKey, signerBump: vaultBump })
				.accounts({
					vault: vaultPublicKey,
				})
				.postInstructions([
					anchor.web3.SystemProgram.transfer({
						fromPubkey: payerPublicKey,
						toPubkey: vaultSigner,
						lamports: 10 * anchor.web3.LAMPORTS_PER_SOL,
					}),
				])
				.signers([payer, vault])
				.rpc();

			console.log("Create Vault Signature: ", createVaultSignature);

			// const vaultAccount = await program.account.vault.fetch(vaultPublicKey);
			// printVaultAccount(vaultAccount);

			const withdrawAmount = 1 * anchor.web3.LAMPORTS_PER_SOL;

			const ix = anchor.web3.SystemProgram.transfer({
				fromPubkey: vaultSigner,
				toPubkey: payerPublicKey,
				lamports: withdrawAmount,
			});

			let { blockhash } = await provider.connection.getLatestBlockhash();
			const proposalAccountSize = calculateTransactionSize([ix], blockhash);
			// console.log("Proposal Account Size: ", proposalAccountSize);

			const proposalParams = {
				name: "Proposal 0001",
				actions: [
					{
						accountSpecs: ix.keys,
						data: ix.data,
						programId: ix.programId,
					},
				],
				proposalAccountSize,
			};
			const timeA = await getBlockTime(provider.connection);
			const createProposalSignature = await program.methods
				.createProposal({
					name: proposalParams.name,
					actions: proposalParams.actions,
					proposalAccountSize: proposalParams.proposalAccountSize,
				})
				.accounts({
					proposal: proposals[0].publicKey,
					vault: vaultPublicKey,
					proposer: payerPublicKey,
				})
				.signers([payer, proposals[0]])
				.rpc();
			const timeB = await getBlockTime(provider.connection);

			const timestamps = new Set(
				Array.from({ length: timeB - timeA + 1 }, (_, i) => (timeA + i).toString()),
			);

			console.log("Create Proposal Signature: ", createProposalSignature);

			let proposalAccount = await program.account.proposal.fetchNullable(
				proposals[0].publicKey,
			);

			assert(proposalAccount != null, "Proposal Account does not exits in chain");
			// printProposalAccount(proposalAccount);

			assert(proposalAccount.name === proposalParams.name, "Proposal names does not match");
			assert(
				timestamps.has(proposalAccount.createdDate.toString()),
				"Proposal created date does not fall within TimeA and TimeB",
			);
			assert(
				proposalAccount.expiryDate.eq(proposalAccount.createdDate.addn(ONE_MONTH)),
				"Expiry date must be one month after creation date",
			);
			assert(!proposalAccount.isExecuted, "Proposal must not be executed during creation");
			assert(JSON.stringify(proposalAccount.proposalStage) == JSON.stringify({ draft: {} }));

			const executionSignature = await program.methods
				.executeProposal()
				.accounts({
					proposal: proposals[0].publicKey,
					caller: payerPublicKey,
				})
				.remainingAccounts([
					...ix.keys.map((key) => ({
						...key,
						isSigner: key.pubkey.equals(vaultSigner) ? false : key.isSigner,
					})),
					{
						pubkey: ix.programId,
						isSigner: false,
						isWritable: false,
					},
				])
				.signers([payer])
				.rpc();

			console.log("Execution signature:", executionSignature);

			proposalAccount = await program.account.proposal.fetchNullable(proposals[0].publicKey);
			// printProposalAccount(proposalAccount);

			assert(proposalAccount != null, "Proposal account does not exists");
			assert(
				proposalAccount.isExecuted,
				"Proposal must be marked as executed after execution",
			);
			assert(
				JSON.stringify(proposalAccount.proposalStage) === JSON.stringify({ completed: {} }),
				"Proposal must be marked as executed after execution",
			);
		});
	});
});
