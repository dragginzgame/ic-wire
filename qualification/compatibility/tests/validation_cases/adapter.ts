// Consumer-owned runtime adapter: generated modules select this through their
// existing runtime_module parameter. No global registry or generated validator.
import {createPayloadCodec as payload,createCodec as frame} from './codec-runtime.ts';
export const calls:string[]=[];
const checks:Readonly<Record<string,(value:unknown)=>void>>=Object.fromEntries(
  ['root','alias','item','leaf'].map(name=>[name,(_value:unknown)=>{calls.push(name);}])
);
export function createPayloadCodec(meta:unknown,types:unknown) {return payload(meta,types,checks);}
export function createCodec(meta:unknown,types:unknown) {return frame(meta,types,checks);}
