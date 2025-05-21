import { assert } from 'chai';

import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';

import { AacsVault } from '../target/types/aacs_vault';
import { getBlockTime } from './shared';

describe("Create vault flow", () => {
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

	describe("flow: createVault()", () => {
		it("creates vault", async () => {
			// Add your test here.
			const timeA = await getBlockTime(provider.connection);

			const createVaultSignature = await program.methods
				.createVault({ owner: payerPublicKey, signerBump: vaultBump })
				.accounts({
					vault: vaultPublicKey,
				})
				.signers([vault, payer])
				.rpc();

			const timeB = await getBlockTime(provider.connection);

			console.log("Create Vault Signature: ", createVaultSignature);

			const vaultAccount = await program.account.vault.fetchNullable(vaultPublicKey);

			assert(vaultAccount != null, "Vault account does not exists");
			// printVaultAccount(vaultAccount);

			assert(vaultAccount.owner.equals(payerPublicKey), "Payer is not vault owner");
			assert(vaultAccount.signerBump === vaultBump, "Vualt bump does not match");

			const timestamps: string[] = Array.from({ length: timeB + 1 - timeA }, (_, i) =>
				(timeA + i).toString(),
			);

			assert(
				timestamps.includes(vaultAccount.createdDate.toString()),
				"Created date does not fall within TimeA and TimeB range",
			);
		});
	});
});
