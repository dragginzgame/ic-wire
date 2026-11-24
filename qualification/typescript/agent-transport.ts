import type { HttpAgent } from '@icp-sdk/core/agent';
import type { Transport } from '../target/service/service.ts';

export function agentTransport(agent: HttpAgent): Transport<string> {
  return {
    async query(canisterId, methodName, arg) {
      const response = await agent.query(canisterId, { methodName, arg });
      if (response.status === 'rejected') throw new Error(response.reject_message);
      return response.reply.arg;
    },
    async update(canisterId, methodName, arg) {
      const response = await agent.update(canisterId, { methodName, arg });
      return response.reply;
    },
  };
}
