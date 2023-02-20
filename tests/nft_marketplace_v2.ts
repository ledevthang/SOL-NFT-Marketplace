import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { NftMarketplaceV2 } from "../target/types/nft_marketplace_v2";
import { Keypair, PublicKey } from "@solana/web3.js";
import { ASSOCIATED_TOKEN_PROGRAM_ID, createAssociatedTokenAccount, createMint, getOrCreateAssociatedTokenAccount, mintTo, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { createKeypairFromFile } from "./utils";
import { BN } from "bn.js";
import { findProgramAddressSync } from "@coral-xyz/anchor/dist/cjs/utils/pubkey";
import { expect } from "chai";
const { SystemProgram } = anchor.web3;

describe("nft_marketplace_v2", () => {
    // Configure the client to use the local cluster.
    const provider = anchor.AnchorProvider.local(
        // "https://solana-devnet.g.alchemy.com/v2/wRDcdu07s9RATBEH4sdhTA8P6FQNeWJh"
    );
    anchor.setProvider(provider);

    const program = anchor.workspace.NftMarketplaceV2 as Program<NftMarketplaceV2>;

    let owner;
    let tokenAccount1;
    let tokenAccount2;
    let tokenAccount3;
    let minterNft1;
    let minterNft2;
    let minterNft3;
    let buyer;
    let state_account;
    it("Init state!", async () => {
        owner = await createKeypairFromFile(__dirname + "/../../../../tungleanh/.config/solana/id.json");

        state_account = await createKeypairFromFile(__dirname + "/../../../my-solana-wallet/state_account.json");
        await program.methods.initState(10).accounts({
            stateAccount: state_account.publicKey,
            user: owner.publicKey,
            systemProgram: SystemProgram.programId
        }).signers([state_account]).rpc()

        let state = (await program.account.state.fetch(state_account.publicKey)).ownerCut;
        expect(state).to.be.equal(10);
    });

    it("Mint", async () => {
        owner = await createKeypairFromFile(__dirname + "/../../../../tungleanh/.config/solana/id.json");

        minterNft1 = await createMint(provider.connection, owner, owner.publicKey, null, 0)
        minterNft2 = await createMint(provider.connection, owner, owner.publicKey, null, 0)
        minterNft3 = await createMint(provider.connection, owner, owner.publicKey, null, 0)

        buyer = await createKeypairFromFile(__dirname + "/../../../my-solana-wallet/my-keypair.json");
        await provider.connection.requestAirdrop(buyer.publicKey, 1e9);

        tokenAccount1 = await createAssociatedTokenAccount(
            provider.connection,
            owner,
            minterNft1,
            owner.publicKey
        )

        await mintTo(
            provider.connection,
            owner,
            minterNft1,
            tokenAccount1,
            owner.publicKey,
            1
        )

        tokenAccount2 = await createAssociatedTokenAccount(
            provider.connection,
            owner,
            minterNft2,
            owner.publicKey
        )

        await mintTo(
            provider.connection,
            owner,
            minterNft2,
            tokenAccount2,
            owner.publicKey,
            1
        )

        tokenAccount3 = await createAssociatedTokenAccount(
            provider.connection,
            owner,
            minterNft3,
            owner.publicKey
        )

        await mintTo(
            provider.connection,
            owner,
            minterNft3,
            tokenAccount3,
            owner.publicKey,
            1
        )
    })

    it("Create listing", async () => {
        buyer = await createKeypairFromFile(__dirname + "/../../../my-solana-wallet/my-keypair.json");
        let id_1 = 1;
        let id_2 = 2;
        let id_3 = 3;
        let id_4 = 4;

        const [pda_account_owner_id_1, bump_account_owner_id_1] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from(owner.publicKey.toBuffer()), Buffer.from(id_1.toString())],
            program.programId
        );

        const [pda_account_owner_id_2, bump_account_owner_id_2] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from(owner.publicKey.toBuffer()), Buffer.from(id_2.toString())],
            program.programId
        );

        const [pda_account_owner_id_3, bump_account_owner_id_3] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from(owner.publicKey.toBuffer()), Buffer.from(id_3.toString())],
            program.programId
        );

        let tokenAccountOfProgram1 = await getOrCreateAssociatedTokenAccount(
            provider.connection,
            owner,
            minterNft1,
            pda_account_owner_id_1,
            true
        )

        let tokenAccountOfProgram2 = await getOrCreateAssociatedTokenAccount(
            provider.connection,
            owner,
            minterNft2,
            pda_account_owner_id_2,
            true
        )

        let tokenAccountOfProgram3 = await getOrCreateAssociatedTokenAccount(
            provider.connection,
            owner,
            minterNft3,
            pda_account_owner_id_3,
            true
        )

        await program.methods.createListing(id_1.toString(), new BN(10), minterNft1, new BN(1675761459), new BN(1675761459), false).accounts({
            from: tokenAccount1,
            to: tokenAccountOfProgram1.address,
            listingAccount: pda_account_owner_id_1,
            tokenProgram: TOKEN_PROGRAM_ID,
            user: owner.publicKey,
            systemProgram: SystemProgram.programId
        }).rpc()

        await program.methods.createListing(id_2.toString(), new BN(10), minterNft2, new BN(1675761459), new BN(1675761459), true).accounts({
            from: tokenAccount2,
            to: tokenAccountOfProgram2.address,
            listingAccount: pda_account_owner_id_2,
            tokenProgram: TOKEN_PROGRAM_ID,
            user: owner.publicKey,
            systemProgram: SystemProgram.programId
        }).rpc()

        await program.methods.createListing(id_3.toString(), new BN(10), minterNft3, new BN(1675761459), new BN(1675761459), true).accounts({
            from: tokenAccount3,
            to: tokenAccountOfProgram3.address,
            listingAccount: pda_account_owner_id_3,
            tokenProgram: TOKEN_PROGRAM_ID,
            user: owner.publicKey,
            systemProgram: SystemProgram.programId
        }).rpc()

        // let balance1 = await provider.connection.getTokenAccountBalance(
        // 	tokenAccountOfProgram1.address
        // );

        await program.methods.setPrice(id_1.toString(), new BN(100000000)).accounts({
            listingAccount: pda_account_owner_id_1,
            user: owner.publicKey,
            systemProgram: SystemProgram.programId
        }).rpc()

        await program.methods.bid(id_2.toString(), new BN(10000)).accounts({
            listingAccount: pda_account_owner_id_2,
            user: buyer.publicKey,
            systemProgram: SystemProgram.programId,
            ownerAuction: owner.publicKey
        }).signers([buyer]).rpc()

        await program.methods.cancelListing(id_2.toString(), bump_account_owner_id_2).accounts({
            listingAccount: pda_account_owner_id_2,
            user: owner.publicKey,
            systemProgram: SystemProgram.programId,
            auth: pda_account_owner_id_2,
            from: tokenAccountOfProgram2.address,
            to: tokenAccount2,
            tokenProgram: TOKEN_PROGRAM_ID,
        }).signers([owner]).rpc()

        const buyerTokenAddress = await anchor.utils.token.associatedAddress({
            mint: minterNft1,
            owner: buyer.publicKey,
        });

        let bal = await provider.connection.getBalance(owner.publicKey);
        console.log(bal);

        await program.methods.buyNft(id_1.toString(), bump_account_owner_id_1).accounts({
            listingAccount: pda_account_owner_id_1,
            user: buyer.publicKey,
            systemProgram: SystemProgram.programId,
            auth: pda_account_owner_id_1,
            fromTokenAccount: tokenAccountOfProgram1.address,
            toTokenAccount: buyerTokenAddress,
            mint: minterNft1,
            owner: owner.publicKey,
            seller: owner.publicKey,
            stateAccount: state_account.publicKey,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID
        }).signers([buyer]).rpc()

        bal = await provider.connection.getBalance(owner.publicKey);
        console.log(bal);

        const [pda_account_buyer_id_4, bump_account_buyer_id_4] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from(buyer.publicKey.toBuffer()), Buffer.from(id_4.toString())],
            program.programId
        );

        let tokenAccountOfProgram4 = await getOrCreateAssociatedTokenAccount(
            provider.connection,
            owner,
            minterNft1,
            pda_account_owner_id_3,
            true
        )

        await program.methods.createListing(id_4.toString(), new BN(10), minterNft3, new BN(1675761459), new BN(1675761459), true).accounts({
            from: buyerTokenAddress,
            to: tokenAccountOfProgram4.address,
            listingAccount: pda_account_buyer_id_4,
            tokenProgram: TOKEN_PROGRAM_ID,
            user: buyer.publicKey,
            systemProgram: SystemProgram.programId
        }).signers([buyer]).rpc()
    })
});
