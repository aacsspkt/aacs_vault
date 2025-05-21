import { assert } from 'chai';

import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';

import { AacsVault } from '../target/types/aacs_vault';
import { sleep } from './shared';

describe("Deposit, and withdraw flow", () => {
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

	describe("flow: createVault() -> depositSol() -> withdrawSol()", () => {
		it("works as expected", async () => {
			const createVaultSignature = await program.methods
				.createVault({ owner: payerPublicKey, signerBump: vaultBump })
				.accounts({
					vault: vaultPublicKey,
				})
				.signers([vault, payer])
				.rpc();

			console.log("Create Vault Signature: ", createVaultSignature);

			// const vaultAccount = await program.account.vault.fetch(vaultPublicKey);
			// printVaultAccount(vaultAccount);

			const vaultBalanceA = await provider.connection.getBalance(vaultSigner);
			// console.log(
			// 	"Vault Balance Before Deposit:",
			// 	BigNumber(vaultBalanceA).div(anchor.web3.LAMPORTS_PER_SOL).toFixed(),
			// );
			const depositAmount = 10 * anchor.web3.LAMPORTS_PER_SOL;
			const depositSolSignature = await program.methods
				.depositSol({ amount: new anchor.BN(depositAmount) })
				.accounts({
					payer: payerPublicKey,
					vault: vaultPublicKey,
				})
				.signers([payer])
				.rpc();

			console.log("Deposit Signature: ", depositSolSignature);

			await sleep(1000);
			const vaultBalanceB = await provider.connection.getBalance(vaultSigner);
			// console.log(
			// 	"Vault Balance After Deposit:",
			// 	BigNumber(vaultBalanceB).div(anchor.web3.LAMPORTS_PER_SOL).toFixed(),
			// );

			assert(
				vaultBalanceB - vaultBalanceA === 10 * anchor.web3.LAMPORTS_PER_SOL,
				"Vault balance after deposit does not match",
			);

			const withdrawAmount = 1 * anchor.web3.LAMPORTS_PER_SOL;
			const withdrawSolSignature = await program.methods
				.withdrawSol({ amount: new anchor.BN(withdrawAmount) })
				.accounts({
					vault: vaultPublicKey,
					recipient: payerPublicKey,
				})
				.signers([payer])
				.rpc();

			console.log("Withdraw Signature: ", withdrawSolSignature);

			await sleep(1000);
			const vaultBalanceC = await provider.connection.getBalance(vaultSigner);
			// console.log(
			// 	"Vault Balance After Withdraw: ",
			// 	BigNumber(vaultBalanceC).div(anchor.web3.LAMPORTS_PER_SOL).toFixed(),
			// );

			assert(
				vaultBalanceB - vaultBalanceC === 1 * anchor.web3.LAMPORTS_PER_SOL,
				"Vault balance after withdraw does not match",
			);
		});
	});
});
