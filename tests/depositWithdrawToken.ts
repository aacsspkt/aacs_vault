import { BigNumber } from 'bignumber.js';
import { assert } from 'chai';

import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import {
  getAssociatedTokenAddressSync,
  getOrCreateAssociatedTokenAccount,
} from '@solana/spl-token';

import { AacsVault } from '../target/types/aacs_vault';
import {
  createNewMint,
  sleep,
} from './shared';

describe("Deposit and withdraw Token flow", () => {
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

	before(async () => {});

	describe("flow: createVault() -> depositToken() -> withdrawToken()", () => {
		it("works as expected", async () => {
			const mintDecimals = 9;
			const UNITS_PER_TOKEN = BigNumber(10).pow(mintDecimals);
			const mintKeypair = anchor.web3.Keypair.generate();
			const mint = await createNewMint(provider.connection, payer, mintKeypair, mintDecimals);

			await sleep(3000);

			const payerAta = getAssociatedTokenAddressSync(mint, payerPublicKey, true);
			// const payerTokenBalance = await provider.connection.getTokenAccountBalance(payerAta);
			// console.log("PayerTokenBalance:", payerTokenBalance.value.uiAmountString);

			const createVaultSignature = await program.methods
				.createVault({ owner: payerPublicKey, signerBump: vaultBump })
				.accounts({
					vault: vaultPublicKey,
				})
				.signers([vault, payer])
				.rpc();

			console.log("\tCreate Vault Signature: ", createVaultSignature);

			// const vaultAccount = await program.account.vault.fetch(vaultPublicKey);
			// printVaultAccount(vaultAccount);

			const vaultSignerTokenAccount = await getOrCreateAssociatedTokenAccount(
				provider.connection,
				payer,
				mint,
				vaultSigner,
				true,
			);

			const vaultSignerAta = vaultSignerTokenAccount.address;

			const vaultBalanceA = await provider.connection.getTokenAccountBalance(vaultSignerAta);
			// console.log(
			// 	"Vault Balance Before Deposit:",
			// 	BigNumber(vaultBalanceA).div(anchor.web3.LAMPORTS_PER_SOL).toFixed(),
			// );
			const depositAmount = BigNumber(10).times(UNITS_PER_TOKEN).toFixed(0);
			const depositTokenSignature = await program.methods
				.depositToken({
					amount: new anchor.BN(depositAmount),
					decimals: mintDecimals,
				})
				.accountsPartial({
					depositor: payerPublicKey,
					vault: vaultPublicKey,
					tokenMint: mint,
					depositorTokenAccount: payerAta,
					vaultSigner: vaultSigner,
					vaultSignerTokenAccount: vaultSignerAta,
				})
				.signers([payer])
				.rpc();

			console.log("\tDeposit Signature: ", depositTokenSignature);

			await sleep(1000);
			const vaultBalanceB = await provider.connection.getTokenAccountBalance(vaultSignerAta);
			// console.log(
			// 	"Vault Balance After Deposit:",
			// 	BigNumber(vaultBalanceB).div(anchor.web3.LAMPORTS_PER_SOL).toFixed(),
			// );

			assert(
				BigNumber(vaultBalanceB.value.amount)
					.minus(vaultBalanceA.value.amount)
					.isEqualTo(BigNumber(10).times(UNITS_PER_TOKEN)),
				"Vault token balance after deposit does not match",
			);

			const withdrawAmount = BigNumber(3).times(UNITS_PER_TOKEN).toFixed();
			const withdrawTokenSignature = await program.methods
				.withdrawToken({
					amount: new anchor.BN(withdrawAmount),
					decimals: mintDecimals,
				})
				.accountsPartial({
					vault: vaultPublicKey,
					withdrawer: payerPublicKey,
					tokenMint: mint,
					// vaultSigner: vaultSigner,
					// vaultSignerTokenAccount: vaultSignerAta,
					// withdrawerTokenAccount: payerAta,
				})
				.signers([payer])
				.rpc();

			console.log("\tWithdraw Signature: ", withdrawTokenSignature);

			await sleep(1000);
			const vaultBalanceC = await provider.connection.getTokenAccountBalance(vaultSignerAta);
			// console.log(
			// 	"Vault Balance After Withdraw: ",
			// 	BigNumber(vaultBalanceC).div(anchor.web3.LAMPORTS_PER_SOL).toFixed(),
			// );

			assert(
				BigNumber(vaultBalanceB.value.amount)
					.minus(withdrawAmount)
					.isEqualTo(vaultBalanceC.value.amount),
				"Vault token balance after withdraw does not match",
			);
		});
	});
});
