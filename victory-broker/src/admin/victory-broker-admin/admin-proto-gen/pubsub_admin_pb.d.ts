import * as jspb from 'google-protobuf'



export class AdapterRequest extends jspb.Message {
  getHz(): number;
  setHz(value: number): AdapterRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): AdapterRequest.AsObject;
  static toObject(includeInstance: boolean, msg: AdapterRequest): AdapterRequest.AsObject;
  static serializeBinaryToWriter(message: AdapterRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): AdapterRequest;
  static deserializeBinaryFromReader(message: AdapterRequest, reader: jspb.BinaryReader): AdapterRequest;
}

export namespace AdapterRequest {
  export type AsObject = {
    hz: number,
  }
}

export class AdapterResponse extends jspb.Message {
  getAdaptersList(): Array<Adapter>;
  setAdaptersList(value: Array<Adapter>): AdapterResponse;
  clearAdaptersList(): AdapterResponse;
  addAdapters(value?: Adapter, index?: number): Adapter;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): AdapterResponse.AsObject;
  static toObject(includeInstance: boolean, msg: AdapterResponse): AdapterResponse.AsObject;
  static serializeBinaryToWriter(message: AdapterResponse, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): AdapterResponse;
  static deserializeBinaryFromReader(message: AdapterResponse, reader: jspb.BinaryReader): AdapterResponse;
}

export namespace AdapterResponse {
  export type AsObject = {
    adaptersList: Array<Adapter.AsObject>,
  }
}

export class Adapter extends jspb.Message {
  getName(): string;
  setName(value: string): Adapter;

  getLive(): boolean;
  setLive(value: boolean): Adapter;

  getDescription(): string;
  setDescription(value: string): Adapter;

  getStatsList(): Array<string>;
  setStatsList(value: Array<string>): Adapter;
  clearStatsList(): Adapter;
  addStats(value: string, index?: number): Adapter;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): Adapter.AsObject;
  static toObject(includeInstance: boolean, msg: Adapter): Adapter.AsObject;
  static serializeBinaryToWriter(message: Adapter, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): Adapter;
  static deserializeBinaryFromReader(message: Adapter, reader: jspb.BinaryReader): Adapter;
}

export namespace Adapter {
  export type AsObject = {
    name: string,
    live: boolean,
    description: string,
    statsList: Array<string>,
  }
}

export class ChannelRequest extends jspb.Message {
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ChannelRequest.AsObject;
  static toObject(includeInstance: boolean, msg: ChannelRequest): ChannelRequest.AsObject;
  static serializeBinaryToWriter(message: ChannelRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ChannelRequest;
  static deserializeBinaryFromReader(message: ChannelRequest, reader: jspb.BinaryReader): ChannelRequest;
}

export namespace ChannelRequest {
  export type AsObject = {
  }
}

export class ChannelResponse extends jspb.Message {
  getChannelsList(): Array<PubSubChannel>;
  setChannelsList(value: Array<PubSubChannel>): ChannelResponse;
  clearChannelsList(): ChannelResponse;
  addChannels(value?: PubSubChannel, index?: number): PubSubChannel;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ChannelResponse.AsObject;
  static toObject(includeInstance: boolean, msg: ChannelResponse): ChannelResponse.AsObject;
  static serializeBinaryToWriter(message: ChannelResponse, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ChannelResponse;
  static deserializeBinaryFromReader(message: ChannelResponse, reader: jspb.BinaryReader): ChannelResponse;
}

export namespace ChannelResponse {
  export type AsObject = {
    channelsList: Array<PubSubChannel.AsObject>,
  }
}

export class PubSubChannel extends jspb.Message {
  getTopic(): string;
  setTopic(value: string): PubSubChannel;

  getSubscribersList(): Array<string>;
  setSubscribersList(value: Array<string>): PubSubChannel;
  clearSubscribersList(): PubSubChannel;
  addSubscribers(value: string, index?: number): PubSubChannel;

  getPublishersList(): Array<string>;
  setPublishersList(value: Array<string>): PubSubChannel;
  clearPublishersList(): PubSubChannel;
  addPublishers(value: string, index?: number): PubSubChannel;

  getMessageCount(): number;
  setMessageCount(value: number): PubSubChannel;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): PubSubChannel.AsObject;
  static toObject(includeInstance: boolean, msg: PubSubChannel): PubSubChannel.AsObject;
  static serializeBinaryToWriter(message: PubSubChannel, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): PubSubChannel;
  static deserializeBinaryFromReader(message: PubSubChannel, reader: jspb.BinaryReader): PubSubChannel;
}

export namespace PubSubChannel {
  export type AsObject = {
    topic: string,
    subscribersList: Array<string>,
    publishersList: Array<string>,
    messageCount: number,
  }
}

