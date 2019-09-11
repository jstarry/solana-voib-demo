import React, {useRef, useEffect, FunctionComponent} from 'react';

import Gatekeeper from './gatekeeper';
import PrepayService from './prepay';

const Stream: FunctionComponent<any> = ({webRtc}) => {
  const videoEl = useRef(null);

  useEffect((): any => {
    let transport: any;
    let stream: any;

    (async (): Promise<any> => {
      const prepayService = new PrepayService();
      await prepayService.init();
      const contractKey = await prepayService.newContract();
      const data = await webRtc.consumerData();

      // remove udp candidate
      data.iceCandidates.shift();

      const candidate = data.iceCandidates[0];
      const destination = `${candidate.ip}:${candidate.port}`;
      console.log('DESTINATION', destination);

      const [ip, port] = await Gatekeeper.newConnection(
        `${candidate.ip}:${candidate.port}`,
        contractKey.toString(),
        prepayService.getPayerKey().toString(),
      );

      // replace tcp candidate with gatekeeper connection
      candidate.ip = ip;
      candidate.port = port;

      const newDestination = `${candidate.ip}:${candidate.port}`;
      console.log('NEW DESTINATION', newDestination);

      transport = webRtc.consumerTransport(data);
      stream = await webRtc.consumeStream(transport);
      videoEl.current.srcObject = stream;
    })();

    return (): void => {
      if (transport) transport.close();
      if (stream) {
        stream.getTracks().forEach((track: any): void => track.stop());
      }
    };
  }, []);

  return <video autoPlay playsInline ref={videoEl} />;
};

export default Stream;
