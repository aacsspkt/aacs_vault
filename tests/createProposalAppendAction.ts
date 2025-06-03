import { assert } from 'chai';

import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';

import { AacsVault } from '../target/types/aacs_vault';
import {
  calculateTransactionSize,
  getBlockTime,
} from './shared';

describe("Create proposal and append action flow", () => {
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

	describe("flow: createVault() -> createProposal() -> appendAction()", () => {
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
			assert(JSON.stringify(proposalAccount.proposalStage) == JSON.stringify({ draft: {} }));
			assert(!proposalAccount.isExecuted, "Proposal must not be executed during creation");
			assert(
				proposalAccount.vault.equals(vaultPublicKey),
				"Proposal does not belong to right vault",
			);
			assert((proposalAccount.actions.length = 1), "Proposal must have 1 action");

			const proposalAccountInfo = await provider.connection.getAccountInfo(
				proposals[0].publicKey,
				"confirmed",
			);
			assert(proposalAccountInfo, "Proposal account is not initialized.");
			const fetchedProposalAccountSize = proposalAccountInfo.data.length;

			console.log(
				"set proposal size: %d & fetched proposal size: %d",
				proposalAccountSize,
				fetchedProposalAccountSize,
			);

			const ix2 = anchor.web3.SystemProgram.transfer({
				fromPubkey: vaultSigner,
				toPubkey: payerPublicKey,
				lamports: withdrawAmount,
			});

			let lbh = await provider.connection.getLatestBlockhash();
			const proposalAccountSize2 =
				calculateTransactionSize([ix], lbh.blockhash) + fetchedProposalAccountSize - 8; // Subtract Discriminator

			const appendActionSignature = await program.methods
				.appendActions({
					actions: [
						{
							accountSpecs: ix2.keys,
							data: ix2.data,
							programId: ix2.programId,
						},
					],
					proposalAccountSize: proposalAccountSize2,
				})
				.accounts({
					proposal: proposals[0].publicKey,
					proposer: payerPublicKey,
				})
				.signers([payer])
				.rpc();

			console.log("Append Action Signature: ", appendActionSignature);

			proposalAccount = await program.account.proposal.fetchNullable(proposals[0].publicKey);

			assert(proposalAccount != null, "Proposal Account does not exits in chain");

			// printProposalAccount(proposalAccount);

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
			assert(
				proposalAccount.vault.equals(vaultPublicKey),
				"Proposal does not belong to right vault",
			);
			assert((proposalAccount.actions.length = 2), "Proposal must have 2 action");
		});
	});
});
