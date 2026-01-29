/**
 * Initialize Computation Definitions on Devnet
 *
 * Run: npx ts-node scripts/init-comp-defs.ts
 */

import * as anchor from "@coral-xyz/anchor"
import { Program } from "@coral-xyz/anchor"
import { PublicKey, Keypair } from "@solana/web3.js"
import * as fs from "fs"
import * as os from "os"
import { SipArciumTransfer } from "../target/types/sip_arcium_transfer"
import {
  getArciumEnv,
  getMXEAccAddress,
  getCompDefAccAddress,
  getCompDefAccOffset,
} from "@arcium-hq/client"

const CLUSTER_OFFSET = 456 // Devnet v0.6.3 cluster

async function main() {
  // Setup provider
  const connection = new anchor.web3.Connection(
    "https://api.devnet.solana.com",
    "confirmed"
  )

  // Load keypair
  const keypairPath = `${os.homedir()}/.config/solana/id.json`
  const keypairData = JSON.parse(fs.readFileSync(keypairPath, "utf-8"))
  const wallet = Keypair.fromSecretKey(Uint8Array.from(keypairData))

  console.log("Wallet:", wallet.publicKey.toBase58())

  // Setup Anchor
  const provider = new anchor.AnchorProvider(
    connection,
    new anchor.Wallet(wallet),
    { commitment: "confirmed" }
  )
  anchor.setProvider(provider)

  // Load program
  const programId = new PublicKey("S1P5q5497A6oRCUutUFb12LkNQynTNoEyRyUvotmcX9")
  const idl = JSON.parse(
    fs.readFileSync("./target/idl/sip_arcium_transfer.json", "utf-8")
  )
  const program = new Program(idl, provider) as Program<SipArciumTransfer>

  console.log("Program:", program.programId.toBase58())

  // Get MXE account
  const mxeAccount = getMXEAccAddress(program.programId)
  console.log("MXE Account:", mxeAccount.toBase58())

  // Initialize computation definitions
  const compDefs = [
    "private_transfer",
    "check_balance",
    "validate_swap",
  ]

  for (const name of compDefs) {
    const offsetBytes = getCompDefAccOffset(name)
    const offset = Buffer.from(offsetBytes).readUInt32LE()
    const compDefAccount = getCompDefAccAddress(program.programId, offset)

    console.log(`\nInitializing ${name}...`)
    console.log(`  Offset: ${offset}`)
    console.log(`  Account: ${compDefAccount.toBase58()}`)

    try {
      // Check if already initialized
      const info = await connection.getAccountInfo(compDefAccount)
      if (info) {
        console.log(`  Already initialized, skipping`)
        continue
      }

      // Initialize based on the computation name
      const methodName = `init${name.split('_').map(w => w[0].toUpperCase() + w.slice(1)).join('')}CompDef`

      const tx = await (program.methods as any)[methodName]()
        .accounts({
          payer: wallet.publicKey,
          mxeAccount,
          compDefAccount,
        })
        .rpc()

      console.log(`  Initialized: ${tx}`)
    } catch (err: any) {
      console.error(`  Error: ${err.message}`)
    }
  }

  console.log("\nDone!")
}

main().catch(console.error)
