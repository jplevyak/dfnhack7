export const idlFactory = ({ IDL }) => {
  const ClearArguments = IDL.Record({});
  const BatchId = IDL.Nat;
  const Key = IDL.Text;
  const CreateAssetArguments = IDL.Record({
    'key' : Key,
    'content_type' : IDL.Text,
  });
  const UnsetAssetContentArguments = IDL.Record({
    'key' : Key,
    'content_encoding' : IDL.Text,
  });
  const DeleteAssetArguments = IDL.Record({ 'key' : Key });
  const ChunkId = IDL.Nat;
  const SetAssetContentArguments = IDL.Record({
    'key' : Key,
    'sha256' : IDL.Opt(IDL.Vec(IDL.Nat8)),
    'chunk_ids' : IDL.Vec(ChunkId),
    'content_encoding' : IDL.Text,
  });
  const BatchOperationKind = IDL.Variant({
    'CreateAsset' : CreateAssetArguments,
    'UnsetAssetContent' : UnsetAssetContentArguments,
    'DeleteAsset' : DeleteAssetArguments,
    'SetAssetContent' : SetAssetContentArguments,
    'Clear' : ClearArguments,
  });
  const RecordResult = IDL.Record({
    'expires' : IDL.Nat64,
    'link' : IDL.Text,
    'canister_id' : IDL.Opt(IDL.Text),
    'description' : IDL.Text,
  });
  const UpdatedRecordResult = IDL.Record({
    'link' : IDL.Text,
    'canister_id' : IDL.Opt(IDL.Text),
  });
  const HeaderField = IDL.Tuple(IDL.Text, IDL.Text);
  const HttpRequest = IDL.Record({
    'url' : IDL.Text,
    'method' : IDL.Text,
    'body' : IDL.Vec(IDL.Nat8),
    'headers' : IDL.Vec(HeaderField),
  });
  const Token = IDL.Record({});
  const StreamingCallbackHttpResponse = IDL.Record({
    'token' : IDL.Opt(Token),
    'body' : IDL.Vec(IDL.Nat8),
  });
  const StreamingStrategy = IDL.Variant({
    'Callback' : IDL.Record({
      'token' : Token,
      'callback' : IDL.Func(
          [Token],
          [StreamingCallbackHttpResponse],
          ['query'],
        ),
    }),
  });
  const HttpResponse = IDL.Record({
    'body' : IDL.Vec(IDL.Nat8),
    'headers' : IDL.Vec(HeaderField),
    'streaming_strategy' : IDL.Opt(StreamingStrategy),
    'status_code' : IDL.Nat16,
  });
  return IDL.Service({
    'authorize' : IDL.Func([IDL.Principal], [], []),
    'claim_link' : IDL.Func(
        [
          IDL.Record({
            'link' : IDL.Text,
            'canister_id' : IDL.Opt(IDL.Text),
            'description' : IDL.Opt(IDL.Text),
          }),
        ],
        [],
        [],
      ),
    'clear' : IDL.Func([ClearArguments], [], []),
    'commit_batch' : IDL.Func(
        [
          IDL.Record({
            'batch_id' : BatchId,
            'operations' : IDL.Vec(BatchOperationKind),
          }),
        ],
        [],
        [],
      ),
    'create_asset' : IDL.Func([CreateAssetArguments], [], []),
    'create_batch' : IDL.Func(
        [IDL.Record({})],
        [IDL.Record({ 'batch_id' : BatchId })],
        [],
      ),
    'create_chunk' : IDL.Func(
        [IDL.Record({ 'content' : IDL.Vec(IDL.Nat8), 'batch_id' : BatchId })],
        [IDL.Record({ 'chunk_id' : ChunkId })],
        [],
      ),
    'delete_asset' : IDL.Func([DeleteAssetArguments], [], []),
    'get' : IDL.Func(
        [IDL.Record({ 'key' : Key, 'accept_encodings' : IDL.Vec(IDL.Text) })],
        [
          IDL.Record({
            'content' : IDL.Vec(IDL.Nat8),
            'sha256' : IDL.Opt(IDL.Vec(IDL.Nat8)),
            'content_type' : IDL.Text,
            'content_encoding' : IDL.Text,
            'total_length' : IDL.Nat,
          }),
        ],
        ['query'],
      ),
    'get_chunk' : IDL.Func(
        [
          IDL.Record({
            'key' : Key,
            'sha256' : IDL.Opt(IDL.Vec(IDL.Nat8)),
            'index' : IDL.Nat,
            'content_encoding' : IDL.Text,
          }),
        ],
        [IDL.Record({ 'content' : IDL.Vec(IDL.Nat8) })],
        ['query'],
      ),
    'get_link' : IDL.Func([IDL.Text], [IDL.Opt(RecordResult)], ['query']),
    'get_links' : IDL.Func([], [IDL.Vec(RecordResult)], ['query']),
    'get_updated_links' : IDL.Func(
        [IDL.Nat64],
        [IDL.Vec(UpdatedRecordResult)],
        ['query'],
      ),
    'http_request' : IDL.Func([HttpRequest], [HttpResponse], ['query']),
    'http_request_stream_callback' : IDL.Func(
        [IDL.Opt(Token)],
        [StreamingCallbackHttpResponse],
        ['query'],
      ),
    'list' : IDL.Func(
        [IDL.Record({})],
        [
          IDL.Vec(
            IDL.Record({
              'key' : Key,
              'encodings' : IDL.Vec(
                IDL.Record({
                  'sha256' : IDL.Opt(IDL.Vec(IDL.Nat8)),
                  'length' : IDL.Nat,
                  'content_encoding' : IDL.Text,
                })
              ),
              'content_type' : IDL.Text,
            })
          ),
        ],
        ['query'],
      ),
    'search' : IDL.Func([IDL.Text], [IDL.Vec(RecordResult)], ['query']),
    'set_asset_content' : IDL.Func([SetAssetContentArguments], [], []),
    'set_link_reserved' : IDL.Func(
        [
          IDL.Record({
            'expires' : IDL.Nat64,
            'link' : IDL.Text,
            'canister_id' : IDL.Opt(IDL.Text),
            'description' : IDL.Opt(IDL.Text),
          }),
        ],
        [],
        [],
      ),
    'store' : IDL.Func(
        [
          IDL.Record({
            'key' : Key,
            'content' : IDL.Vec(IDL.Nat8),
            'sha256' : IDL.Opt(IDL.Vec(IDL.Nat8)),
            'content_type' : IDL.Text,
            'content_encoding' : IDL.Text,
          }),
        ],
        [],
        [],
      ),
    'unset_asset_content' : IDL.Func([UnsetAssetContentArguments], [], []),
  });
};
export const init = ({ IDL }) => { return []; };
