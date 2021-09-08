import type { Principal } from '@dfinity/principal';
export type BatchId = bigint;
export type ChunkId = bigint;
export type HeaderField = [string, string];
export interface HttpRequest {
  'url' : string,
  'method' : string,
  'body' : Array<number>,
  'headers' : Array<HeaderField>,
}
export interface HttpResponse {
  'body' : Array<number>,
  'headers' : Array<HeaderField>,
  'streaming_strategy' : [] | [StreamingStrategy],
  'status_code' : number,
}
export type Key = string;
export interface Record {
  'created' : bigint,
  'owner' : [] | [Principal],
  'link' : string,
  'description' : string,
  'datum' : [] | [Array<number>],
}
export interface RecordResult {
  'created' : bigint,
  'owner' : [] | [Principal],
  'link' : string,
  'description' : string,
}
export interface StreamingCallbackHttpResponse {
  'token' : [] | [Token],
  'body' : Array<number>,
}
export type StreamingStrategy = {
    'Callback' : { 'token' : Token, 'callback' : [Principal, string] }
  };
export type Token = {};
export interface UpdatedRecordResult {
  'link' : string,
  'canister_id' : [] | [string],
}
export interface _SERVICE {
  'authorize' : (arg_0: Principal) => Promise<undefined>,
  'clear' : () => Promise<undefined>,
  'get_data' : () => Promise<Array<RecordResult>>,
  'get_datum' : (arg_0: string) => Promise<[] | [RecordResult]>,
  'http_request' : (arg_0: HttpRequest) => Promise<HttpResponse>,
  'http_request_stream_callback' : (arg_0: [] | [Token]) => Promise<
      StreamingCallbackHttpResponse
    >,
  'notarize' : (arg_0: Principal, arg_1: Array<number>) => Promise<
      [] | [RecordResult]
    >,
  'search' : (arg_0: string) => Promise<Array<RecordResult>>,
}
