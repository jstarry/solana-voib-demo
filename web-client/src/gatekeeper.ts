// eslint-disable-next-line
const jayson = require('jayson/lib/client/browser');
// eslint-disable-next-line
const fetch = require('node-fetch');

function createRpcRequest(url: string): any {
  const server = jayson(
    async (request: any, callback: any): Promise<any> => {
      const options = {
        method: 'POST',
        body: request,
        headers: {
          'Content-Type': 'application/json',
        },
      };

      try {
        const res = await fetch(url, options);
        const text = await res.text();
        callback(null, text);
      } catch (err) {
        callback(err);
      }
    },
  );

  return (method: string, args: any): Promise<any> => {
    return new Promise((resolve: any, reject: any): void => {
      server.request(method, args, (err: any, response: any): void => {
        if (err) {
          reject(err);
          return;
        }
        resolve(response);
      });
    });
  };
}

const GATEKEEPER_IP = '127.0.0.1';
const GATEKEEPER_PORT = 8122;

const rpcRequest = createRpcRequest(
  `http://${GATEKEEPER_IP}:${GATEKEEPER_PORT}`,
);

export default class Gatekeeper {
  static async newConnection(
    destination: string,
    contractKey: string,
    initiatorKey: string,
  ): Promise<[string, number]> {
    const response = await rpcRequest('newConnection', {
      destination,
      contract_pubkey: contractKey, // eslint-disable-line
      initiator_pubkey: initiatorKey, // eslint-disable-line
    });

    return [GATEKEEPER_IP, parseInt(response.result.port)];
  }
}
