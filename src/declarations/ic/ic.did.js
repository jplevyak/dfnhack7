export const idlFactory = ({ IDL }) => {
  const RecordResult = IDL.Record({
    'created' : IDL.Nat64,
    'owner' : IDL.Opt(IDL.Principal),
    'link' : IDL.Text,
    'description' : IDL.Text,
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
    'clear' : IDL.Func([], [], []),
    'get_data' : IDL.Func([], [IDL.Vec(RecordResult)], ['query']),
    'get_datum' : IDL.Func([IDL.Text], [IDL.Opt(RecordResult)], ['query']),
    'http_request' : IDL.Func([HttpRequest], [HttpResponse], ['query']),
    'http_request_stream_callback' : IDL.Func(
        [IDL.Opt(Token)],
        [StreamingCallbackHttpResponse],
        ['query'],
      ),
    'notarize' : IDL.Func(
        [IDL.Principal, IDL.Vec(IDL.Nat8)],
        [IDL.Opt(RecordResult)],
        [],
      ),
    'search' : IDL.Func([IDL.Text], [IDL.Vec(RecordResult)], ['query']),
  });
};
export const init = ({ IDL }) => { return []; };
