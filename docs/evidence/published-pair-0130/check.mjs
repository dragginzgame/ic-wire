import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { principal } from './principal.js';
const vectors = JSON.parse(readFileSync(new URL('./vectors.json', import.meta.url)));
for (const { raw, request, response } of vectors.valid) {
  const value = { caller: Uint8Array.from(raw) };
  assert.deepEqual([...principal.encodeRequest(value)], request);
  assert.deepEqual([...principal.encodeResponse(value)], response);
  assert.deepEqual(principal.decodeRequest(Uint8Array.from(request)).caller, value.caller);
  assert.deepEqual(principal.decodeResponse(Uint8Array.from(response)).caller, value.caller);
}
for (const { raw, request, response } of vectors.invalid) {
  const value = { caller: Uint8Array.from(raw) };
  assert.throws(() => principal.encodeRequest(value));
  assert.throws(() => principal.encodeResponse(value));
  assert.throws(() => principal.decodeRequest(Uint8Array.from(request)));
  assert.throws(() => principal.decodeResponse(Uint8Array.from(response)));
}
console.log('Principal Rust/TypeScript frames and length refusals passed');

const { account } = await import('./account.js');
const accounts = JSON.parse(readFileSync(new URL('./accounts.json', import.meta.url)));
for (const { owner, subaccount, request, response } of accounts) {
  const value = { account: {
    owner: Uint8Array.from(owner),
    subaccount: subaccount === null ? null : { some: subaccount },
  } };
  assert.deepEqual([...account.encodeRequest(value)], request);
  assert.deepEqual([...account.encodeResponse(value)], response);
  for (const decoded of [account.decodeRequest(Uint8Array.from(request)),
                         account.decodeResponse(Uint8Array.from(response))]) {
    assert.deepEqual(decoded.account.owner, value.account.owner);
    if (subaccount === null) assert.equal(decoded.account.subaccount, null);
    else assert.deepEqual(decoded.account.subaccount.some, subaccount);
  }
}
for (const length of [0, 31, 33]) {
  assert.throws(() => account.encodeRequest({ account: {
    owner: new Uint8Array([4]), subaccount: { some: Array(length).fill(0) },
  } }));
}
console.log('Account Rust/TypeScript frames preserve absent, zero and nonzero subaccounts');
const invalidAccounts = JSON.parse(readFileSync(new URL('./accounts-invalid.json', import.meta.url)));
for (const { name, request, response } of invalidAccounts) {
  assert.throws(() => account.decodeRequest(Uint8Array.from(request)), name);
  assert.throws(() => account.decodeResponse(Uint8Array.from(response)), name);
}
console.log('Account malformed request/response refusals agree with Rust');
