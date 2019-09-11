import React, {useRef, useEffect, FunctionComponent} from 'react';

const Broadcast: FunctionComponent<any> = ({webRtc}) => {
  const videoEl = useRef(null);

  useEffect((): any => {
    let transport: any;
    let stream: any;

    (async function(): Promise<any> {
      const transport = await webRtc.producerTransport();
      const stream = await webRtc.getUserMedia(transport);
      videoEl.current.srcObject = stream;
    })();

    return (): void => {
      if (transport) transport.stop();
      if (stream) {
        stream.getTracks().forEach((track: any): void => track.stop());
      }
    };
  }, []);

  return <video autoPlay playsInline ref={videoEl} />;
};

export default Broadcast;
