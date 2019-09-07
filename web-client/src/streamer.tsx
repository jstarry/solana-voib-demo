import React, {useRef, useEffect, FunctionComponent} from 'react';

const Streamer: FunctionComponent<{}> = () => {
  const videoEl = useRef(null);
  const mediaStreamConstraints = {
    video: true,
  };

  // Handles success by adding the MediaStream to the video element.
  function gotLocalMediaStream(mediaStream: MediaStream): void {
    if (videoEl.current) {
      videoEl.current.srcObject = mediaStream;
    }
  }

  // Handles error by logging a message to the console with the error message.
  function handleLocalMediaStreamError(error: Error): void {
    console.error('navigator.getUserMedia error: ', error);
  }

  useEffect((): void => {
    if (navigator.mediaDevices) {
      navigator.mediaDevices
        .getUserMedia(mediaStreamConstraints)
        .then(gotLocalMediaStream)
        .catch(handleLocalMediaStreamError);
    } else {
      alert('No media devices detected');
    }
  });

  return <video autoPlay ref={videoEl} />;
};

export default Streamer;
