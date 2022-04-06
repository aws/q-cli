import { ServerOriginatedMessage, ClientOriginatedMessage } from './fig.pb';

import { b64ToBytes, bytesToBase64 } from './utils';

interface GlobalAPIError {
  error: string;
}

const FigGlobalErrorOccurred = 'FigGlobalErrorOccurred';
const FigProtoMessageRecieved = 'FigProtoMessageRecieved';

type shouldKeepListening = boolean;

export type APIResponseHandler = (
  response: ServerOriginatedMessage['submessage']
) => shouldKeepListening | void;

let messageId = 0;
const handlers: Record<number, APIResponseHandler> = {};

export function setHandlerForId(handler: APIResponseHandler, id: number) {
  handlers[id] = handler;
}

export function sendMessage(
  message: ClientOriginatedMessage['submessage'],
  handler?: APIResponseHandler
) {
  const request: ClientOriginatedMessage = {
    id: (messageId += 1),
    submessage: message
  };

  if (handler && request.id) {
    handlers[request.id] = handler;
  }

  const buffer = ClientOriginatedMessage.encode(request).finish();
  const b64 = bytesToBase64(buffer);

  // eslint-disable-next-line @typescript-eslint/ban-ts-comment
  // @ts-ignore
  if (!window.__TAURI__ || !window.__TAURI__.invoke) {
    console.error(
      'Cannot send request. Fig.js is not supported in this browser.'
    );
    return;
  }

  // eslint-disable-next-line @typescript-eslint/ban-ts-comment
  // @ts-ignore
  window.__TAURI__.invoke('handle_api_request', {
    clientOriginatedMessageB64: b64
  });

  // TODO: Make crossplatform
  return;

  // eslint-disable-next-line @typescript-eslint/ban-ts-comment
  // @ts-ignore
  if (!window.webkit) {
    console.warn(
      'Cannot send request. Fig.js is not supported in this browser.'
    );
    return;
  }

  // eslint-disable-next-line @typescript-eslint/ban-ts-comment
  // @ts-ignore
  if (!window.webkit.messageHandlers.proto) {
    console.error(
      'This version of Fig does not support using protocol buffers. Please update.'
    );
    return;
  }
  // eslint-disable-next-line @typescript-eslint/ban-ts-comment
  // @ts-ignore
  window.webkit.messageHandlers.proto.postMessage(b64);
}

const recievedMessage = (response: ServerOriginatedMessage): void => {
  if (response.id === undefined) {
    return;
  }

  const handler = handlers[response.id];

  if (!handler) {
    return;
  }

  const keepListeningOnID = handlers[response.id](response.submessage);

  if (!keepListeningOnID) {
    delete handlers[response.id];
  }
};

const setupEventListeners = (): void => {
  document.addEventListener(FigGlobalErrorOccurred, (event: Event) => {
    const response = (event as CustomEvent).detail as GlobalAPIError;
    console.error(response.error);
  });

  document.addEventListener(FigProtoMessageRecieved, (event: Event) => {
    const raw = (event as CustomEvent).detail as string;

    const bytes = b64ToBytes(raw);

    const message = ServerOriginatedMessage.decode(bytes);

    recievedMessage(message);
  });
};

async function setupTauriEventListeners() {
  // eslint-disable-next-line @typescript-eslint/ban-ts-comment
  // @ts-ignore
  await window.__TAURI__.event.listen(FigGlobalErrorOccurred, (event: any) => {
    const response = { error: event.payload } as GlobalAPIError;
    console.error(response);
  });

  // eslint-disable-next-line @typescript-eslint/ban-ts-comment
  // @ts-ignore
  await window.__TAURI__.event.listen(FigProtoMessageRecieved, (event: any) => {
    const raw = event.payload as string;

    const bytes = b64ToBytes(raw);

    const message = ServerOriginatedMessage.decode(bytes);

    recievedMessage(message);
  });
}

// We want this to be run automatically
console.log('[fig] setting up event listeners...');
setupEventListeners();
setupTauriEventListeners();
