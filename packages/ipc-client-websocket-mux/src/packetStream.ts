import { Packet, PacketSchema } from "@aws/amazon-q-developer-cli-proto/mux";
import { PacketReader } from "./reader.js";
import { Socket } from "./socket.js";
import { toBinary } from "@bufbuild/protobuf";

export class PacketStream {
  private socket: Socket;
  private readable: PacketReader;

  constructor(socket: Socket) {
    this.socket = socket;
    this.readable = new PacketReader(socket);
  }

  setSocket(socket: Socket) {
    this.socket = socket;
    this.readable.setSocket(socket);
  }

  onPacket(listener: (packet: Packet) => void | Promise<void>) {
    this.readable.onPacket(listener);
  }

  write(packet: Packet) {
    const bytes = toBinary(PacketSchema, packet);
    const base64 = btoa(String.fromCharCode(...bytes));
    this.socket.send(base64 + "\n");
  }
}
