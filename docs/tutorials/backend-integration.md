# Backend Integration

This tutorial covers automating TipJar contract interactions from a server-side application using Node.js and `@stellar/stellar-sdk`.

## Prerequisites

- Node.js 18+
- A deployed and initialized TipJar contract (see [Getting Started](./getting-started.md))
- A funded Stellar account for the backend service

## Install Dependencies

```bash
npm install @stellar/stellar-sdk
```

## Configuration

Store sensitive values in environment variables, never in source code.

```typescript
// config.ts
export const config = {
  rpcUrl: process.env.STELLAR_RPC_URL ?? 'https://soroban-testnet.stellar.org',
  networkPassphrase: process.env.STELLAR_NETWORK ?? 'Test SDF Network ; September 2015',
  contractId: process.env.CONTRACT_ID!,
  tokenAddress: process.env.TOKEN_ADDRESS!,
  // Load the secret key from a secrets manager in production
  serviceSecretKey: process.env.SERVICE_SECRET_KEY!,
};
```

## TipJar Client

Wrap the contract calls in a reusable client class.

```typescript
import {
  Contract,
  Keypair,
  SorobanRpc,
  TransactionBuilder,
  Address,
  nativeToScVal,
  scValToNative,
} from '@stellar/stellar-sdk';
import { config } from './config';

export class TipJarClient {
  private server: SorobanRpc.Server;
  private contract: Contract;
  private keypair: Keypair;

  constructor() {
    this.server = new SorobanRpc.Server(config.rpcUrl);
    this.contract = new Contract(config.contractId);
    this.keypair = Keypair.fromSecret(config.serviceSecretKey);
  }

  async tip(creatorAddress: string, amount: bigint): Promise<string> {
    return this.invoke('tip', [
      new Address(this.keypair.publicKey()).toScVal(),
      new Address(creatorAddress).toScVal(),
      new Address(config.tokenAddress).toScVal(),
      nativeToScVal(amount, { type: 'i128' }),
    ]);
  }

  async getWithdrawableBalance(
    creatorAddress: string,
    tokenAddress: string,
  ): Promise<bigint> {
    const result = await this.query('get_withdrawable_balance', [
      new Address(creatorAddress).toScVal(),
      new Address(tokenAddress).toScVal(),
    ]);
    return scValToNative(result) as bigint;
  }

  async getTotalTips(creatorAddress: string, tokenAddress: string): Promise<bigint> {
    const result = await this.query('get_total_tips', [
      new Address(creatorAddress).toScVal(),
      new Address(tokenAddress).toScVal(),
    ]);
    return scValToNative(result) as bigint;
  }

  private async invoke(method: string, args: unknown[]): Promise<string> {
    const account = await this.server.getAccount(this.keypair.publicKey());
    const tx = new TransactionBuilder(account, {
      fee: '100',
      networkPassphrase: config.networkPassphrase,
    })
      .addOperation(this.contract.call(method, ...args))
      .setTimeout(30)
      .build();

    const simResult = await this.server.simulateTransaction(tx);
    if (SorobanRpc.Api.isSimulationError(simResult)) {
      throw new Error(`Simulation error: ${simResult.error}`);
    }

    const preparedTx = SorobanRpc.assembleTransaction(tx, simResult).build();
    preparedTx.sign(this.keypair);

    const sendResult = await this.server.sendTransaction(preparedTx);
    if (sendResult.status === 'ERROR') {
      throw new Error(`Submit error: ${JSON.stringify(sendResult.errorResult)}`);
    }

    return this.waitForConfirmation(sendResult.hash);
  }

  private async query(method: string, args: unknown[]) {
    const account = await this.server.getAccount(this.keypair.publicKey());
    const tx = new TransactionBuilder(account, {
      fee: '100',
      networkPassphrase: config.networkPassphrase,
    })
      .addOperation(this.contract.call(method, ...args))
      .setTimeout(30)
      .build();

    const simResult = await this.server.simulateTransaction(tx);
    if (SorobanRpc.Api.isSimulationError(simResult)) {
      throw new Error(`Query error: ${simResult.error}`);
    }
    return simResult.result!.retval;
  }

  private async waitForConfirmation(hash: string): Promise<string> {
    for (let i = 0; i < 30; i++) {
      const result = await this.server.getTransaction(hash);
      if (result.status !== SorobanRpc.Api.GetTransactionStatus.NOT_FOUND) {
        if (result.status === SorobanRpc.Api.GetTransactionStatus.FAILED) {
          throw new Error(`Transaction ${hash} failed`);
        }
        return hash;
      }
      await new Promise((r) => setTimeout(r, 1000));
    }
    throw new Error(`Transaction ${hash} not confirmed after 30s`);
  }
}
```

## Batch Tipping

Use `tip_batch` to fan out tips to multiple creators in one transaction (up to 50).

```typescript
import { xdr, nativeToScVal, Address } from '@stellar/stellar-sdk';

interface BatchEntry {
  creator: string;
  token: string;
  amount: bigint;
}

async function sendBatchTips(client: TipJarClient, entries: BatchEntry[]) {
  if (entries.length > 50) {
    throw new Error('Batch size cannot exceed 50');
  }

  const tipsScVal = xdr.ScVal.scvVec(
    entries.map((e) =>
      xdr.ScVal.scvMap([
        new xdr.ScMapEntry({
          key: xdr.ScVal.scvSymbol('creator'),
          val: new Address(e.creator).toScVal(),
        }),
        new xdr.ScMapEntry({
          key: xdr.ScVal.scvSymbol('token'),
          val: new Address(e.token).toScVal(),
        }),
        new xdr.ScMapEntry({
          key: xdr.ScVal.scvSymbol('amount'),
          val: nativeToScVal(e.amount, { type: 'i128' }),
        }),
      ]),
    ),
  );

  // Use the client's invoke method with the batch args
  // (extend TipJarClient.invoke to accept raw ScVal args as needed)
  console.log('Batch tips submitted');
}
```

## Polling for Events

Monitor tip events to trigger downstream actions (e.g., notifications, analytics).

```typescript
async function pollTipEvents(
  server: SorobanRpc.Server,
  contractId: string,
  fromLedger: number,
) {
  const events = await server.getEvents({
    startLedger: fromLedger,
    filters: [
      {
        type: 'contract',
        contractIds: [contractId],
      },
    ],
  });

  for (const event of events.events) {
    const topicSymbol = event.topic[0]; // "tip", "withdraw", etc.
    console.log(`Event: ${topicSymbol} at ledger ${event.ledger}`);
  }

  return events.latestLedger;
}
```

## Retry Logic

Wrap submissions with exponential backoff for transient RPC failures.

```typescript
async function withRetry<T>(
  fn: () => Promise<T>,
  maxAttempts = 3,
): Promise<T> {
  for (let attempt = 1; attempt <= maxAttempts; attempt++) {
    try {
      return await fn();
    } catch (err) {
      if (attempt === maxAttempts) throw err;
      const delay = 500 * 2 ** (attempt - 1);
      console.warn(`Attempt ${attempt} failed, retrying in ${delay}ms`);
      await new Promise((r) => setTimeout(r, delay));
    }
  }
  throw new Error('unreachable');
}

// Usage
await withRetry(() => client.tip(creatorAddress, 1000000n));
```
