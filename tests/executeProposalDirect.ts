import { assert } from 'chai';

import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';

import { AacsVault } from '../target/types/aacs_vault';
import { sleep } from './shared';

describe("Create vault, deposit, and execute proposal direct", () => {
	// Configure the client to use the local cluster.
	anchor.setProvider(anchor.AnchorProvider.env());
	const provider = anchor.getProvider();
	const program = anchor.workspace.aacsVault as Program<AacsVault>;

	const VAULT_SIGNER_PREFIX = anchor.utils.bytes.utf8.encode("aacs_vault_signer");
	const payer = provider.wallet!.payer!;
	const payerPublicKey = payer.publicKey;
	const vault = anchor.web3.Keypair.generate();
	const vaultPublicKey = vault.publicKey;
	const [vaultSigner, vaultBump] = anchor.web3.PublicKey.findProgramAddressSync(
		[VAULT_SIGNER_PREFIX, vaultPublicKey.toBuffer()],
		program.programId,
	);

	describe("flow: createVault() ->  executeProposalDirect()", () => {
		it("works as expected", async () => {
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
				.signers([vault, payer])
				.rpc();

			console.log("Create Vault Signature: ", createVaultSignature);

			const withdrawAmount = 1 * anchor.web3.LAMPORTS_PER_SOL;

			const ix = anchor.web3.SystemProgram.transfer({
				fromPubkey: vaultSigner,
				toPubkey: payerPublicKey,
				lamports: withdrawAmount,
			});

			const params = {
				name: "Proposal 0001",
				actions: [
					{
						accountSpecs: ix.keys,
						data: ix.data,
						programId: ix.programId,
					},
				],
			};

			const vaultBalanceA = await provider.connection.getBalance(vaultSigner);
			// console.log(
			// 	"Vault Balance Before Deposit:",
			// 	BigNumber(vaultBalanceA).div(anchor.web3.LAMPORTS_PER_SOL).toFixed(),
			// );
			const executionSignature = await program.methods
				.executeProposalDirect(params)
				.accounts({
					vault: vaultPublicKey,
					proposer: payerPublicKey,
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

			await sleep(1000);
			const vaultBalanceB = await provider.connection.getBalance(vaultSigner);
			// console.log(
			// 	"Vault Balance After Withdraw:",
			// 	BigNumber(vaultBalanceB).div(anchor.web3.LAMPORTS_PER_SOL).toFixed(),
			// );
			assert(
				vaultBalanceA - vaultBalanceB === 1 * anchor.web3.LAMPORTS_PER_SOL,
				"Vault balance after withdraw does not match",
			);
		});
	});
});
