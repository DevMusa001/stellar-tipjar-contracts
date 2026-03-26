# Frontend Integration

This tutorial shows how to interact with the TipJar contract from a browser-based application using `@stellar/stellar-sdk`.

## Prerequisites

- Node.js 18+
- A deployed and initialized TipJar contract (see [Getting Started](./getting-started.md))
- A wallet that supports Stellar (e.g., Freighter)

## Install Dependencies

```bash
npm install @stellar/stellar-sdk
```

## Connect to the Network

```typescript
import { SorobanRpc, Networks } from '@stellar/stellar-sdk';

const server = new SorobanRpc.Server('https://soroban-testnet.stellar.org');
const networkPassphrase = Networks.TESTNET;
```

## Build and Submit a Transaction

All state-changing contract calls follow the same pattern: build → simulate → sign → submit.

```typescript
import {
  Contract,
  TransactionBuilder,
  nativeToScVal,
  Address,
  xdr,
  SorobanRpc,
  Networks,
  Keypair,
} from '@stellar/stellar-sdk';

const CONTRACT_ID = 'YOUR_CONTRACT_ID';
const TOKEN_ADDRESS = 'YOUR_TOKEN_ADDRESS';

async function sendTip(
  senderKeypair: Keypair,
  creatorAddress: string,
  amount: bigint,
) {
  const contract = new Contract(CONTRACT_ID);
  const account = await server.getAccount(senderKeypair.publicKey());

  const tx = new TransactionBuilder(account, {
    fee: '100',
    networkPassphrase,
  })
    .addOperation(
      contract.call(
        'tip',
        new Address(senderKeypair.publicKey()).toScVal(),
        new Address(creatorAddress).toScVal(),
        new Address(TOKEN_ADDRESS).toScVal(),
        nativeToScVal(amount, { type: 'i128' }),
      ),
    )
    .setTimeout(30)
    .build();

  // Simulate to get the footprint and resource fees
  const simResult = await server.simulateTransaction(tx);
  if (SorobanRpc.Api.isSimulationError(simResult)) {
    throw new Error(`Simulation failed: ${simResult.error}`);
  }

  const preparedTx = SorobanRpc.assembleTransaction(tx, simResult).build();
  preparedTx.sign(senderKeypair);

  const sendResult = await server.sendTransaction(preparedTx);
  if (sendResult.status === 'ERROR') {
    throw new Error(`Submit failed: ${sendResult.errorResult}`);
  }

  // Poll for confirmation
  return pollForResult(sendResult.hash);
}

async function pollForResult(hash: string) {
  let result = await server.getTransaction(hash);
  while (result.status === SorobanRpc.Api.GetTransactionStatus.NOT_FOUND) {
    await new Promise((r) => setTimeout(r, 1000));
    result = await server.getTransaction(hash);
  }
  if (result.status === SorobanRpc.Api.GetTransactionStatus.FAILED) {
    throw new Error('Transaction failed');
  }
  return result;
}
```

## Read-Only Queries

Read-only calls (prefixed `get_`) do not require signing and can be simulated directly.

```typescript
import { scValToNative } from '@stellar/stellar-sdk';

async function getWithdrawableBalance(
  creatorAddress: string,
  tokenAddress: string,
): Promise<bigint> {
  const contract = new Contract(CONTRACT_ID);
  const account = await server.getAccount(creatorAddress);

  const tx = new TransactionBuilder(account, {
    fee: '100',
    networkPassphrase,
  })
    .addOperation(
      contract.call(
        'get_withdrawable_balance',
        new Address(creatorAddress).toScVal(),
        new Address(tokenAddress).toScVal(),
      ),
    )
    .setTimeout(30)
    .build();

  const simResult = await server.simulateTransaction(tx);
  if (SorobanRpc.Api.isSimulationError(simResult)) {
    throw new Error(`Query failed: ${simResult.error}`);
  }

  return scValToNative(simResult.result!.retval) as bigint;
}
```

## Listening for Events

Subscribe to tip events to update your UI in real time.

```typescript
async function watchTipEvents(creatorAddress: string) {
  // Fetch recent ledgers and scan for tip events
  const latestLedger = await server.getLatestLedger();

  const events = await server.getEvents({
    startLedger: latestLedger.sequence - 1000,
    filters: [
      {
        type: 'contract',
        contractIds: [CONTRACT_ID],
        topics: [
          ['*', new Address(creatorAddress).toScVal().toXDR('base64')],
        ],
      },
    ],
  });

  for (const event of events.events) {
    const [, , sender, amount] = event.value.map(scValToNative);
    console.log(`Tip received: ${amount} from ${sender}`);
  }
}
```

## Freighter Wallet Integration

When running in a browser, use Freighter to sign transactions instead of a raw keypair.

```typescript
import freighterApi from '@stellar/freighter-api';

async function sendTipWithFreighter(
  creatorAddress: string,
  amount: bigint,
) {
  const { address: senderAddress } = await freighterApi.getAddress();
  const contract = new Contract(CONTRACT_ID);
  const account = await server.getAccount(senderAddress);

  const tx = new TransactionBuilder(account, {
    fee: '100',
    networkPassphrase,
  })
    .addOperation(
      contract.call(
        'tip',
        new Address(senderAddress).toScVal(),
        new Address(creatorAddress).toScVal(),
        new Address(TOKEN_ADDRESS).toScVal(),
        nativeToScVal(amount, { type: 'i128' }),
      ),
    )
    .setTimeout(30)
    .build();

  const simResult = await server.simulateTransaction(tx);
  if (SorobanRpc.Api.isSimulationError(simResult)) {
    throw new Error(`Simulation failed: ${simResult.error}`);
  }

  const preparedTx = SorobanRpc.assembleTransaction(tx, simResult).build();
  const signedXdr = await freighterApi.signTransaction(preparedTx.toXDR(), {
    networkPassphrase,
  });

  const signedTx = TransactionBuilder.fromXDR(signedXdr, networkPassphrase);
  return server.sendTransaction(signedTx);
}
```

## Error Handling

Map contract error codes to user-friendly messages:

```typescript
const ERROR_MESSAGES: Record<number, string> = {
  1: 'Contract is already initialized.',
  2: 'This token is not supported.',
  3: 'Tip amount must be greater than zero.',
  4: 'No balance available to withdraw.',
  5: 'Message exceeds 280 characters.',
  9: 'You are not authorized to perform this action.',
  11: 'Batch exceeds the 50-tip limit.',
  12: 'Insufficient token balance.',
  13: 'Unlock time must be in the future.',
  14: 'This tip is still time-locked.',
  15: 'Locked tip not found.',
};

function handleContractError(error: unknown): string {
  // Extract the error code from the Soroban error result
  const match = String(error).match(/Error\(Contract, #(\d+)\)/);
  if (match) {
    return ERROR_MESSAGES[Number(match[1])] ?? `Contract error #${match[1]}`;
  }
  return 'An unexpected error occurred.';
}
```
