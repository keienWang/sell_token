import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SellToken } from "../target/types/sell_token";

describe("sell_token", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.sellToken2 as Program<SellToken>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initSaleAccount().rpc();
    console.log("Your transaction signature", tx);
  });
});
