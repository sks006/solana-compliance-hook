import { Connection, PublicKey } from "@solana/web3.js";

/**
 * Verifies if an account has been fully initialized with structural data allocation
 */
export async function accountExists(connection: Connection, address: PublicKey): Promise<boolean> {
  const accountInfo = await connection.getAccountInfo(address);
  return accountInfo !== null && accountInfo.data.length > 0;
}