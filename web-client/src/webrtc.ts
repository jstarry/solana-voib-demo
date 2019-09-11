import * as mediasoup from 'mediasoup-client';

// eslint-disable-next-line
const socketClient = require('socket.io-client');

const MEDIA_SOUP_SERVER_PORT = 3000;
const MEDIA_SOUP_SOCKET_OPTS = {
  path: '/server',
  transports: ['websocket'],
};

// Adds support for Promise to socket.io-client
const socketPromise = function(socket: any) {
  return function request(type: any, data: any = {}): Promise<void> {
    return new Promise((resolve: any): void => {
      socket.emit(type, data, resolve);
    });
  };
};

export default class WebRtcServer {
  socket: any;
  device: any;

  constructor(onConnected: any) {
    const hostname: string = window.location.hostname;
    const serverUrl = `https://${hostname}:${MEDIA_SOUP_SERVER_PORT}`;
    this.socket = socketClient(serverUrl, MEDIA_SOUP_SOCKET_OPTS);
    this.socket.request = socketPromise(this.socket);
    this.socket.on('connect', async () => {
      const data = await this.socket.request('getRouterRtpCapabilities');
      this.device = await WebRtcServer.loadDevice(data);
      onConnected();
    });
  }

  connected(): boolean {
    return !!this.device;
  }

  async producerTransport(): Promise<any> {
    const data = await this.socket.request('createProducerTransport', {
      forceTcp: false,
      rtpCapabilities: this.device.rtpCapabilities,
    });
    if (data.error) {
      console.error(data.error);
      return;
    }

    const transport = this.device.createSendTransport(data);
    transport.on(
      'connect',
      ({dtlsParameters}: any, callback: any, errback: any) => {
        console.log('connect transport');
        this.socket
          .request('connectProducerTransport', {dtlsParameters})
          .then(callback)
          .catch(errback);
      },
    );

    transport.on(
      'produce',
      async ({kind, rtpParameters}: any, callback: any, errback: any) => {
        console.log('produce transport');
        try {
          const {id} = await this.socket.request('produce', {
            transportId: transport.id,
            kind,
            rtpParameters,
          });
          callback({id});
        } catch (err) {
          errback(err);
        }
      },
    );

    transport.on('connectionstatechange', (state: any) => {
      console.log('connectionstatechange transport');
      switch (state) {
        case 'connecting':
          console.log('connecting');
          break;

        case 'connected':
          console.log('connected');
          break;

        case 'failed':
          transport.close();
          console.log('failed');
          break;

        default:
          break;
      }
    });

    return transport;
  }

  async consumeStream(transport: any): Promise<MediaStream> {
    const {rtpCapabilities} = this.device;
    const data = await this.socket.request('consume', {rtpCapabilities});
    const {producerId, id, kind, rtpParameters} = data;

    const codecOptions = {};
    const consumer = await transport.consume({
      id,
      producerId,
      kind,
      rtpParameters,
      codecOptions,
    });
    const stream = new MediaStream();
    stream.addTrack(consumer.track);
    this.socket.request('resume');
    return stream;
  }

  consumerTransport(data: any): any {
    const transport = this.device.createRecvTransport(data);
    transport.on(
      'connect',
      ({dtlsParameters}: any, callback: any, errback: any) => {
        this.socket
          .request('connectConsumerTransport', {
            transportId: transport.id,
            dtlsParameters,
          })
          .then(callback)
          .catch(errback);
      },
    );

    transport.on('connectionstatechange', (state: any) => {
      switch (state) {
        case 'connecting':
          console.log('connecting');
          break;

        case 'connected':
          console.log('connected');
          break;

        case 'failed':
          transport.close();
          console.log('failed');
          break;

        default:
          break;
      }
    });

    return transport;
  }

  async consumerData(): Promise<any> {
    console.log('create consumer transport');
    return await this.socket.request('createConsumerTransport', {
      forceTcp: true,
    });
  }

  async getUserMedia(transport: any): Promise<MediaStream> {
    let stream;
    try {
      stream = await navigator.mediaDevices.getUserMedia({video: true});
    } catch (err) {
      console.error('starting webcam failed,', err.message);
      throw err;
    }
    const track = stream.getVideoTracks()[0];
    await transport.produce({track});
    return stream;
  }

  private static async loadDevice(routerRtpCapabilities: any): Promise<any> {
    try {
      const device = new mediasoup.Device();
      await device.load({routerRtpCapabilities});
      return device;
    } catch (error) {
      if (error.name === 'UnsupportedError') {
        console.error('browser not supported');
      }
    }
  }
}
