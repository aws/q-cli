import type {
  InsertTextRequest,
  InterceptRequest,
  IpcClient,
  RunProcessRequest,
  RunProcessResponse,
  InterceptedKeyHook,
  PostExecHook,
  PreExecHook,
  PromptHook,
} from "@aws/amazon-q-developer-cli-ipc-client-core";
import {
  Clientbound,
  Clientbound_Request,
  Clientbound_RequestSchema,
  Hostbound,
  Hostbound_Request,
  Hostbound_Response,
  PingSchema,
  Pong,
} from "@aws/amazon-q-developer-cli-proto/mux";
import { PacketStream } from "./packetStream.js";
import { CsWebsocket, Socket } from "./socket.js";
import { clientboundToPacket, packetToHostbound } from "./mux.js";
import { EditBufferHook } from "@aws/amazon-q-developer-cli-proto/local";
import Emittery, { UnsubscribeFunction } from "emittery";
import { create } from "@bufbuild/protobuf";

export { CsWebsocket };

const EditBufferHookSymbol = Symbol("EditBufferHook");
const PromptHookSymbol = Symbol("PromptHook");
const PreExecHookSymbol = Symbol("PreExecHook");
const PostExecHookSymbol = Symbol("PostExecHook");
const InterceptedKeyHookSymbol = Symbol("InterceptedKeyHook");

export class WebsocketMuxBackend implements IpcClient {
  emitter: Emittery = new Emittery();
  packetStream: PacketStream;

  constructor(websocket: CsWebsocket) {
    const socket = Socket.cs(websocket);
    console.log("1. socket created");
    this.packetStream = new PacketStream(socket);
    console.log("2. packet stream created");
    this.packetStream.onPacket(async (packet) => {
      console.log("3. packet received");
      const hostbound = await packetToHostbound(packet);
      console.log("4. decoded hostbound", { hostbound });
      this.handleHostbound(hostbound);
      console.log("5. hostbound handled");
    });
  }

  private handleHostbound(message: Hostbound) {
    const submessage = message.submessage;
    console.log(submessage);
    if (submessage.value?.$unknown && submessage.value.$unknown.length > 0) {
      console.warn("Unknown request", submessage.value);
      return;
    }
    switch (submessage?.case) {
      case "request":
        this.handleHostboundRequest(submessage.value);
        break;
      case "response":
        this.handleHostboundResponse(submessage.value);
        break;
      case "pong":
        this.handleHostboundPong(submessage.value);
        break;
      default:
        console.warn("Unknown submessage", submessage);
        break;
    }
  }

  private handleHostboundRequest(request: Hostbound_Request) {
    if (
      request.inner.value?.$unknown &&
      request.inner.value.$unknown.length > 0
    ) {
      console.warn("Unknown request", request.inner);
      return;
    }
    switch (request.inner.case) {
      case "editBuffer":
        this.emitter.emit(EditBufferHookSymbol, request.inner.value);
        break;
      case "interceptedKey":
        this.emitter.emit(InterceptedKeyHookSymbol, request.inner.value);
        break;
      case "postExec":
        this.emitter.emit(PostExecHookSymbol, request.inner.value);
        break;
      case "preExec":
        this.emitter.emit(PreExecHookSymbol, request.inner.value);
        break;
      case "prompt":
        this.emitter.emit(PromptHookSymbol, request.inner.value);
        break;
      default:
        console.warn("Unknown request", request.inner);
        break;
    }
  }

  private handleHostboundResponse(response: Hostbound_Response) {
    switch (response.inner.case) {
      case "runProcess":
        this.emitter.emit(
          `runProcess-${response.messageId}`,
          response.inner.value,
        );
        break;
      default:
        console.warn("Unknown response", response.inner);
        break;
    }
  }

  private handleHostboundPong(pong: Pong) {
    this.emitter.emit(`pong-${pong.messageId}`);
  }

  // Helper requests

  private async sendClientbound(clientbound: Omit<Clientbound, "$typeName">) {
    const packet = await clientboundToPacket(clientbound);
    this.packetStream.write(packet);
  }

  private async sendClientboundRequest(
    sessionId: string,
    messageId: string | undefined,
    clientbound: Clientbound_Request["inner"],
  ) {
    const packet = await clientboundToPacket({
      submessage: {
        case: "request",
        value: create(Clientbound_RequestSchema, {
          sessionId,
          messageId: messageId ?? crypto.randomUUID(),
          inner: clientbound,
        }),
      },
    });
    this.packetStream.write(packet);
  }

  insertText(sessionId: string, request: InsertTextRequest): void {
    console.log("insertText");
    this.sendClientboundRequest(sessionId, undefined, {
      case: "insertText",
      value: request,
    });
  }

  intercept(sessionId: string, request: InterceptRequest): void {
    console.log("intercept");
    this.sendClientboundRequest(sessionId, undefined, {
      case: "intercept",
      value: request,
    });
  }

  async runProcess(
    sessionId: string,
    request: RunProcessRequest,
  ): Promise<RunProcessResponse> {
    const messageId = crypto.randomUUID();
    this.sendClientboundRequest(sessionId, messageId, {
      case: "runProcess",
      value: request,
    });
    return await this.emitter.once(`runProcess-${messageId}`);
  }

  async ping(): Promise<void> {
    const messageId = crypto.randomUUID();
    this.sendClientbound({
      submessage: {
        case: "ping",
        value: create(PingSchema, {
          messageId,
        }),
      },
    });
    await this.emitter.once(`pong-${messageId}`);
  }

  onEditBufferChange(
    callback: (notification: EditBufferHook) => void | Promise<void>,
  ): UnsubscribeFunction {
    return this.emitter.on(EditBufferHookSymbol, callback);
  }

  onPrompt(callback: (notification: PromptHook) => void): UnsubscribeFunction {
    return this.emitter.on(PromptHookSymbol, callback);
  }

  onPreExec(
    callback: (notification: PreExecHook) => void | Promise<void>,
  ): UnsubscribeFunction {
    return this.emitter.on(PreExecHookSymbol, callback);
  }

  onPostExec(
    callback: (notification: PostExecHook) => void | Promise<void>,
  ): UnsubscribeFunction {
    return this.emitter.on(PostExecHookSymbol, callback);
  }

  onInterceptedKey(
    callback: (notification: InterceptedKeyHook) => void | Promise<void>,
  ): UnsubscribeFunction {
    return this.emitter.on(InterceptedKeyHookSymbol, callback);
  }

  async isActive(timeout?: number | undefined) {
    const timeoutPromise = new Promise<void>((_, reject) =>
      setTimeout(() => reject(), timeout ?? 5000),
    );
    const anyEventPromise = new Promise<void>((resolve) =>
      this.emitter.onAny((_) => {
        resolve();
      }),
    );
    try {
      await Promise.race([timeoutPromise, anyEventPromise]);
      return true;
    } catch {
      return false;
    }
  }
}
