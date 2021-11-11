import { v4 as uuidv4 } from 'uuid';
import net from 'net';
import fs from 'fs';

type SocketCallback = (bytes: Buffer) => void;

type Socket = {
  path: string;
  callbacks: Record<string, SocketCallback>;
  server: net.Server;
};

const sockets: Record<string, Socket> = {};

export const socketListen = (
  path: string,
  callback: SocketCallback,
  uuid?: string
): string => {
  const callbackUUID = uuid ?? uuidv4();

  if (sockets[path]) {
    sockets[path].callbacks[callbackUUID] = callback;
    return callbackUUID;
  }

  try {
    fs.unlinkSync(path);
  } catch (e) {
    /* console.log(e) */
  }

  const server = net.createServer();
  sockets[path] = {
    path,
    callbacks: { [callbackUUID]: callback },
    server,
  };

  server.on('connection', s => {
    s.on('end', () => {});

    s.on('data', data => {
      if (sockets[path]) {
        Object.values(sockets[path].callbacks).forEach(cb => cb(data));
      }
      s.end();
    });

    s.on('error', console.log);
  });
  server.listen(path);

  return callbackUUID;
};

export const closeSocket = (path: string) => {
  sockets[path]?.server.close();

  try {
    fs.unlinkSync(path);
  } catch (e) {
    /* console.log(e) */
  }

  delete sockets[path];
};

export const removeListener = (path: string, uuid: string) => {
  delete sockets[path]?.callbacks[uuid];
  if (Object.keys(sockets[path].callbacks).length === 0) {
    closeSocket(path);
  }
};
