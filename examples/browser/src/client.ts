import type { HttpAgent } from '@icp-sdk/core/agent';
import * as apply from '../generated/apply.js';
import * as inspect from '../generated/inspect.js';

// Transport policy stays here; all DTO validation and encoding is generated.
export function createClient(agent: HttpAgent, canisterId: string) {
  return {
    async apply(request: apply.Request): Promise<apply.Response> {
      const result = await agent.update(canisterId, { methodName: apply.methodName, arg: apply.encodeRequest(request) });
      return apply.decodeResponse(result.reply);
    },
    async inspect(request: inspect.Request): Promise<inspect.Response> {
      const result = await agent.query(canisterId, { methodName: inspect.methodName, arg: inspect.encodeRequest(request) });
      if (result.status !== 'replied') throw new Error(`query rejected: ${result.reject_message}`);
      return inspect.decodeResponse(new Uint8Array(result.reply.arg));
    },
  };
}
