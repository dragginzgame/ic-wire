import {createService, type Transport} from './service.ts';
declare const transport: Transport<string>;
const client = createService('canister', transport);
const result: Promise<{ok: boolean} | {err: number}> = client.fetch('abc');
// @ts-expect-error The codec accepts text.
client.lookup(12);
const saved: Promise<{ok: number} | {err: boolean}> = client.save(17);
// @ts-expect-error The write codec accepts a number.
client.save('abc');
// @ts-expect-error The write codec returns a boolean error.
const wrong: Promise<{ok: number} | {err: string}> = client.save(17);
