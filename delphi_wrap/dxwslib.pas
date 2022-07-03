unit dxwslib;

interface
uses Winapi.Windows,System.SysUtils,System.Classes,Vcl.Forms,System.RTLConsts;

type
  TSocketAddrInfo = record
    ip_v6: Boolean; //默认为ipv4,
    port: Word;
    ip: array[0..15] of Byte; //ip地址
    function display: string;
  end;
  PSocketAddrInfo = ^TSocketAddrInfo;

  TAuthShakeRequestHeader = reference to function(header,Value: string): Boolean;

  THandleShakeRequest = record
  private
    req: Pointer;
    fauth: TAuthShakeRequestHeader;
    authOk: Boolean;
    function GetHeaders(header: string): string;
  public
    property Headers[header: string]: string read GetHeaders;
    function RequestPath: string;
    function QueryArgs: string;
    function AuthHeaders(authFunc: TAuthShakeRequestHeader): Boolean; //校验headers
  end;
  PHandleShakeRequest = ^THandleShakeRequest;

  THandleShakeResponse = record
  private
    resp: Pointer;
  public
    procedure SetStatusCode(code: Word); //设置返回的StatusCode
    procedure SetHeader(header,Value: UTF8String);
  end;

  TDxSocketClient = class;

  TStringData = record
    utf8data: PByte;
    len: NativeUInt;
    function Value: string;
  end;
  TWebSocketMsgType = (Msg_Text,Msg_Bin,Msg_Ping,Msg_Pong,Msg_Close,Msg_Frame,Msg_None);
  TCloseCode = (
    CC_Bad = 0,
    CC_Normal=1000,
    CC_Away,
    CC_Protocol,
    CC_Unsupported,
    CC_Status=1005,
    CC_Abnormal,
    CC_Invalid,
    CC_Policy,
    CC_Size,
    CC_Extension,
    CC_Error,
    CC_Restart,
    CC_Again,
    CC_Tls=1015,
    CC_Reserved=1016,
    CC_Iana=3000,
    CC_Library=4000);

  TWebSocketMsg = record
    msgType: Word;
    closeCode: Word;
    data: TStringData;
  end;
  PWebSocketMsg = ^TWebSocketMsg;

  TBeforeHandleShakeEvent = procedure(sender: TObject;peerAddr: PSocketAddrInfo;var succed: Boolean) of object;
  TOnHandleShakeEvent = procedure(sender: TObject;request: THandleShakeRequest;resp: THandleShakeResponse;var succed: Boolean) of object;
  TOnClientConnected = procedure(Client: TDxSocketClient) of object;
  TOnSrvRecvMsg = procedure(sender: TObject;Client: TDxSocketClient; msgType: TWebSocketMsgType;data: TStringData) of object;
  TAfterSendMsg = procedure(client: TDxSocketClient;succeed: Boolean; msg: TWebSocketMsg) of object;
  TOnClientClosed = procedure(Client: TDxSocketClient;closeCode: TCloseCode;code: Word;reason: string) of object;
  TDxWebSocketServer = class
  private
    FServerHandle: Pointer;
    FPort: Word;
    FActive: Boolean;
    FBeforeHandleShake: TBeforeHandleShakeEvent;
    FOnHandleShake: TOnHandleShakeEvent;
    FOnRecvMsg: TOnSrvRecvMsg;
    FOnClientClosed: TOnClientClosed;
    FOnClientConnected: TOnClientConnected;
    FSSLKeyFile: string;
    FSSLCertFile: string;
    FOnAfterSendMsg: TAfterSendMsg;
    procedure SetActive(const Value: Boolean);
    procedure SetPort(const Value: Word);
    procedure SetSSLCertFile(const Value: string);
    procedure SetSSLKeyFile(const Value: string);
  protected
    function DoBeforeHandleShake(peerAddr: PSocketAddrInfo): Boolean;virtual;
    function DoHandleShake(request: THandleShakeRequest;resp: THandleShakeResponse): Boolean;virtual;
    function DoClientConnected(connected: Boolean;socket_write: Pointer;socket_Addr: PSocketAddrInfo): TDxSocketClient;virtual;
    procedure DoRecvWebSocketMsg(msgType: TWebSocketMsgType;data: TStringData;client: TDxSocketClient);virtual;
    procedure DoClientClosed(client: TDxSocketClient;closeCode: TCloseCode;code: Word;const reason: string);virtual;
    procedure AfterSendMsg(succeed: Boolean;msg: TWebSocketMsg;client: TDxSocketClient);virtual;
  public
    property Port: Word read FPort write SetPort;
    property SSLCertFile: string read FSSLCertFile write SetSSLCertFile; //证书
    property SSLKeyFile: string read FSSLKeyFile write SetSSLKeyFile; //证书秘钥
    property Active: Boolean read FActive write SetActive;
    property BeforeHandleShake: TBeforeHandleShakeEvent read FBeforeHandleShake write FBeforeHandleShake;
    property OnHandleShake: TOnHandleShakeEvent read FOnHandleShake write FOnHandleShake;
    property OnRecvMsg: TOnSrvRecvMsg read FOnRecvMsg write FOnRecvMsg;
    property OnClientConnected: TOnClientConnected read FOnClientConnected write FOnClientConnected;
    property OnAfterSendMsg: TAfterSendMsg read FOnAfterSendMsg write FOnAfterSendMsg;
    property OnClientClosed: TOnClientClosed read FOnClientClosed write FOnClientClosed;
  end;


  TRustBuffer = record
  private
    buf: PByte;
    buflen: Integer;
  public
    Position: Integer;
    class function Create(str: UTF8String): TRustBuffer; static;
    function AllocRustBuffer(l: Integer): Boolean;
    function WriteString(st: UTF8String): Integer;
  end;

  TRustStream = class(TCustomMemoryStream)
  private
    FCapacity: NativeInt;
  protected
    procedure SetCapacity(NewCapacity: NativeInt); virtual;
    function Realloc(var NewCapacity: Longint): Pointer; virtual;
    property Capacity: NativeInt read FCapacity write SetCapacity;
  public
    destructor Destroy; override;
    procedure Clear;
    procedure LoadFromStream(Stream: TStream);
    procedure LoadFromFile(const FileName: string);
    procedure SetSize(const NewSize: Int64); override;
    procedure SetSize(NewSize: Longint); override;
    function Write(const Buffer; Count: Longint): Longint; override;
    function Write(const Buffer: TBytes; Offset, Count: Longint): Longint; override;
  end;

  TOnRecvMsg = procedure(sender: TObject;msgType: TWebSocketMsgType;data: TStringData) of object;
  TDxSocketClient = class
  private
    FWriter: Pointer;
    FAddress: TSocketAddrInfo;
    FOnRecvmsg: TOnRecvMsg;
    FOnClosed: TOnClientClosed;
    FOnAfterSendMsg: TAfterSendMsg;
    function GetPeerAddr: string;
  protected
    procedure DoRecvWebSocketMsg(msgType: TWebSocketMsgType;data: TStringData);virtual;
    procedure DoClientClosed(closeCode: TCloseCode;code: Word;const reason: string);virtual;
    procedure AfterSendMsg(succeed: Boolean; msg: TWebSocketMsg);virtual;
    procedure DoConnected;virtual;
    //这个里面的buf，只能使用动态库的alloc分配，不需要释放，动态库内部会自动释放
    function Send(tp: TWebSocketMsgType; buf: PByte;buflen: Integer): Boolean;overload;
  public
    destructor Destroy;override;
    procedure Close;virtual;
    property OnClosed: TOnClientClosed read FOnClosed write FOnClosed;
    property OnRecvmsg: TOnRecvMsg read FOnRecvmsg write FOnRecvmsg;
    property OnAfterSendMsg: TAfterSendMsg read FOnAfterSendMsg write FOnAfterSendMsg;
    //rustBuffer不能重复使用，发送了之后，这个buf就无效了
    function Send(tp: TWebSocketMsgType;var buf: TRustBuffer): Boolean;overload;
    //stream发送调用之后，就会清空内容了
    function Send(tp: TWebSocketMsgType;stream: TRustStream): Boolean;overload;

    function Ping(var buf: TRustBuffer): Boolean;
    function Pong(var buf: TRustBuffer): Boolean;
    function SendText(value: UTF8String): Boolean;
    property PeerAddr: string read GetPeerAddr;
  end;

  TBlockProcedure = procedure;stdcall;
  TLogCallBack = procedure(level: Integer;logdata: TStringData);stdcall;

procedure InitWslib(log: TLogCallBack;const maxThread: Word=0;appMainBlock: Boolean=False; const dllpath: string='');

function CloseCodeFromU16(closeCode: Word): TCloseCode;

var
  alloc: function(size: NativeUInt): Pointer;stdcall;
  realloc: function(p: Pointer;oldSize,newSize: NativeUInt): Pointer;stdcall;
  dealloc: procedure(p: Pointer;size: NativeUInt);stdcall;
implementation

type
  TBeforeHandleShake = function(socket_Addr: PSocketAddrInfo;manager: Pointer): Boolean;stdcall;
  TWebSocketHandleShake = function(req: Pointer;response: Pointer;manager: Pointer): Boolean;stdcall;
  TWebSocketConnectedCallBack = function(connected: Boolean;socket_write: Pointer;socket_Addr: PSocketAddrInfo;manager: Pointer): Pointer;stdcall;

  TWebsocketRecvCallBack = procedure(msgType: Byte;data: TStringData;client,manager: Pointer);stdcall;
  TWebsocketSendCallBack = procedure(succeed: Boolean;msg: TWebSocketMsg;client,manager: Pointer);stdcall;
  TWebSocketCloseCallBack = procedure(code: Word;reason: TStringData;client,manager: Pointer);stdcall;
  TLogLevel = (LLDebug,LLInfo,LLWarn,LLError,LLException,LLPanic);

  TWebsocketServerCallBack = record
    manager: Pointer;
    before_handle_shake: TBeforeHandleShake;
    on_handle_shake: TWebSocketHandleShake;
    on_connected: TWebSocketConnectedCallBack;
    on_recv: TWebsocketRecvCallBack;
    on_send: TWebsocketSendCallBack;
    on_closed: TWebSocketCloseCallBack;
  end;
  //tls的证书以及秘钥
  TTLSFileInfo = record
    certFile: TStringData;
    keyFile: TStringData;
  end;

  TMemoryNode = record
    size: NativeUInt;
    mem: array of Byte;
  end;
  PMemoryNode = ^TMemoryNode;

  TCustomMemoryStreamEx = type TCustomMemoryStream;

  TVisitHeaderCallBack = function(key,value: TStringData;data: Pointer): Boolean;stdcall;


function CloseCodeFromU16(closeCode: Word): TCloseCode;
begin
  case closeCode of
  1000: Result := CC_Normal;
  1001: Result := CC_Away;
  1002: Result := CC_Protocol;
  1003: Result := CC_Unsupported;
  1005: Result := CC_Status;
  1006: Result := CC_Abnormal;
  1007: Result := CC_Invalid;
  1008: Result := CC_Policy;
  1009: Result := CC_Size;
  1010: Result := CC_Extension;
  1011: Result := CC_Error;
  1012: Result := CC_Restart;
  1013: Result := CC_Again;
  1015: Result := CC_Tls;
  1016..2999: Result := CC_Reserved;
  3000..3999: Result := CC_Iana;
  4000..4999: Result := CC_Library;
  else Result := CC_Bad;
  end;
end;

procedure AppBlockMain;stdcall;
begin
  Application.Run;
end;

var
  dllHandle: THandle=0;
  
  init_ws_runtime: procedure(maxthreads: Word;blockfunc: TBlockProcedure;log: TLogCallBack);stdcall;
  finalize_ws_runtime: procedure();stdcall;

  websocket_listen: function(port: Word;callback: TWebsocketServerCallBack;tls: TTLSFileInfo): Pointer;stdcall;
  stop_listen: procedure(listenPointer: Pointer);stdcall;
  visit_headers: procedure(req: Pointer;callback: TVisitHeaderCallBack;data: Pointer);stdcall;
  get_header: procedure(key: TStringData;req: Pointer;var v: TStringData);stdcall;
  request_path: procedure(req: Pointer;var path: TStringData);stdcall;
  request_query: procedure(req: Pointer;var path: TStringData);stdcall;

  set_resp_header: procedure(resp: Pointer;key,Value: TStringData);stdcall;
  set_resp_status: procedure(resp: Pointer;statusCode: Word);stdcall;
  send_data: function(msg: TWebSocketMsg;rawwriter: Pointer): Boolean;stdcall;
  free_writer: procedure(rawwriter: Pointer);stdcall;




function GetMem(Size: NativeInt): Pointer;
var
  p: PMemoryNode;
begin
  if Size > 0 then
  begin
    p := alloc(size+Sizeof(NativeUInt));
    p^.size := Size;
    result := Pointer(NativeUint(p)+Sizeof(NativeUInt))
  end
  else result := nil;
end;

function FreeMem(p: Pointer): NativeInt;
begin
  p := Pointer(NativeUInt(p) - SizeOf(NativeUInt));
  Result := PMemoryNode(p)^.size;
  dealloc(p,Result + SizeOf(NativeUInt));
end;

function ReallocMemory(P: Pointer; Size: NativeInt): Pointer;
begin
  p := Pointer(NativeUInt(p) - SizeOf(NativeUInt));
  p := realloc(p,PMemoryNode(p)^.size+SizeOf(NativeUInt),size+Sizeof(NativeUInt));
  PMemoryNode(p)^.size := Size;
  result := Pointer(NativeUint(p)+Sizeof(NativeUInt));
end;

procedure InitWslib(log: TLogCallBack;const maxThread: Word=0;appMainBlock: Boolean=False; const dllpath: string='');
var
  str: string;
begin
  if dllHandle <> 0 then
    Exit;  
  if (dllpath <> '') and FileExists(dllpath) then
    dllHandle := LoadLibrary(PChar(dllPath))
  else
  begin
    str := ExtractFilePath(ParamStr(0))+'wslib.dll';
    if FileExists(dllpath) then
      dllHandle := LoadLibrary(PChar(str));
  end;       
  if dllHandle <> 0 then
  begin
    init_ws_runtime := GetProcAddress(dllHandle,'init_ws_runtime');
    finalize_ws_runtime := GetProcAddress(dllHandle,'finalize_ws_runtime');
    
    websocket_listen := GetProcAddress(dllHandle,'websocket_listen');
    stop_listen := GetProcAddress(dllHandle,'stop_listen');
    visit_headers := GetProcAddress(dllHandle,'visit_headers');
    get_header := GetProcAddress(dllHandle,'get_header');
    request_path := GetProcAddress(dllHandle,'request_path');
    request_query := GetProcAddress(dllHandle,'request_query');
    set_resp_header := GetProcAddress(dllHandle,'set_resp_header');
    set_resp_status := GetProcAddress(dllHandle,'set_resp_status');
    send_data := GetProcAddress(dllHandle,'send_data');
    free_writer := GetProcAddress(dllHandle,'free_writer');

    alloc := GetProcAddress(dllHandle,'alloc');
    realloc := GetProcAddress(dllHandle,'realloc');
    dealloc := GetProcAddress(dllHandle,'dealloc');

    init_ws_runtime(maxThread,AppBlockMain,log);
  end;
end;
{ TStringData }

function TStringData.Value: string;
var
  L: Integer;
begin
  Result := '';
  L := len;
  if (L = 0) or (utf8data = nil) then
    Exit('');
  SetLength(Result, L);
  L := Utf8ToUnicode(PWideChar(Result), L + 1, PAnsiChar(utf8data), L);
  if L > 0 then
    SetLength(Result, L - 1);
end;

{ TSocketAddrInfo }

function TSocketAddrInfo.display: string;
begin
  if ip_v6 then
  begin

  end
  else Result := Format('%d.%d.%d.%d:%d',[ip[0],ip[1],ip[2],ip[3],port]);
end;

{ THandleShakeRequest }


function doAuthHeader(key,value: TStringData;data: Pointer): Boolean;stdcall;
var
  req: PHandleShakeRequest;
begin
  req := data;
  req.authOk := req.fauth(key.Value,value.Value);
  Result := req.authOk;
end;


function THandleShakeRequest.AuthHeaders(
  authFunc: TAuthShakeRequestHeader): Boolean;
begin
  authOk := True;
  if (req <> nil) and Assigned(authFunc) then
    visit_headers(req,@doAuthHeader,@self);
  Result := authOk;
end;

function THandleShakeRequest.GetHeaders(header: string): string;
var
  keyData,value: TStringData;
  key: AnsiString;
begin
  if req <> nil then
  begin
    Key := Utf8Encode(header);
    keyData.utf8data := @Key[1];
    keyData.len := Length(key);
    get_header(keyData,req,value);
    Result := value.Value;
  end
  else Result := '';
end;

function THandleShakeRequest.QueryArgs: string;
var
  v: TStringData;
begin
  if req <> nil then
  begin
    request_query(req,v);
    result := v.Value;
  end
  else Result := '';
end;

function THandleShakeRequest.RequestPath: string;
var
  v: TStringData;
begin
  if req <> nil then
  begin
    request_path(req,v);
    result := v.Value;
  end
  else Result := '';
end;

{ TDxWebSocketServer }

function BeforeHandleShake(socket_Addr: PSocketAddrInfo;manager: Pointer): Boolean;stdcall;
begin
  Result := TDxWebSocketServer(manager).DoBeforeHandleShake(socket_Addr);
end;

function handleShake(req: Pointer;response: Pointer;manager: Pointer): Boolean;stdcall;
var
  request: THandleShakeRequest;
  resp: THandleShakeResponse;
begin
  request.req := req;
  request.authOk := False;
  resp.resp := response;
  result := TDxWebSocketServer(manager).DoHandleShake(request,resp);
end;

function clientConnected(connected: Boolean;socket_write: Pointer;socket_Addr: PSocketAddrInfo;manager: Pointer): Pointer;stdcall;
begin
  result := TDxWebSocketServer(manager).DoClientConnected(connected,socket_write,socket_Addr);
end;

procedure recvWebSocketMsg(msgType: Byte;data: TStringData;client,manager: Pointer);stdcall;
begin
  TDxWebSocketServer(manager).DoRecvWebSocketMsg(TWebSocketMsgType(msgType),data,client);
end;

procedure afterSendMsg(succeed: Boolean;msg: TWebSocketMsg;client,manager: Pointer);stdcall;
begin
  if NativeInt(msg.data.len) = -1 then
  begin
    msg.data.utf8data := nil;
    msg.data.len := 0;
  end;
  TDxWebSocketServer(manager).AfterSendMsg(succeed,msg,client);
end;

procedure clientClosed(code: Word;reason: TStringData;client,manager: Pointer);stdcall;
begin
  TDxWebSocketServer(manager).DoClientClosed(client,CloseCodeFromU16(code),code,reason.Value);
end;

procedure TDxWebSocketServer.AfterSendMsg(succeed: Boolean; msg: TWebSocketMsg;client: TDxSocketClient);
begin
  if Assigned(FOnAfterSendMsg) then
    FOnAfterSendMsg(client,succeed,msg)
  else client.AfterSendMsg(succeed,msg);
end;

function TDxWebSocketServer.DoBeforeHandleShake(peerAddr: PSocketAddrInfo): Boolean;
begin
  Result := True;
  if Assigned(FBeforeHandleShake) then
    FBeforeHandleShake(self,peerAddr,Result);
end;

procedure TDxWebSocketServer.DoClientClosed(client: TDxSocketClient;
  closeCode: TCloseCode; code: Word; const reason: string);
begin
  AtomicExchange(Client.FWriter,nil);
  if Assigned(FOnClientClosed) then
    FOnClientClosed(client,closeCode,Code,reason)
  else client.DoClientClosed(closeCode,code,reason);
end;

function TDxWebSocketServer.DoClientConnected(connected: Boolean;
  socket_write: Pointer; socket_Addr: PSocketAddrInfo): TDxSocketClient;
var
  Client: TDxSocketClient;
begin
  if connected then
  begin
    Client := TDxSocketClient.Create;
    AtomicExchange(Client.FWriter,socket_write);
    Move(socket_Addr^,Client.FAddress,SizeOf(Client.FAddress));
    Result := Client;
    if Assigned(FOnClientConnected) then
      FOnClientConnected(Result)
    else result.DoConnected;
  end
  else
  begin
    Result := nil;
  end;
end;

function TDxWebSocketServer.DoHandleShake(request: THandleShakeRequest;
  resp: THandleShakeResponse): Boolean;
begin
  Result := True;
  if Assigned(FOnHandleShake) then
    FOnHandleShake(Self,request,resp,Result);
end;

procedure TDxWebSocketServer.DoRecvWebSocketMsg(msgType: TWebSocketMsgType;
  data: TStringData; client: TDxSocketClient);
begin
  if Assigned(FOnRecvMsg) then
    FOnRecvMsg(self,client,msgType,data)
  else client.DoRecvWebSocketMsg(msgType,data);
end;

procedure TDxWebSocketServer.SetActive(const Value: Boolean);
var
  callback: TWebsocketServerCallBack;
  tls: TTLSFileInfo;
  st: AnsiString;
begin
  if FActive <> Value then
  begin
    if Value then
    begin
      callback.manager := self;
      callback.before_handle_shake := dxwslib.BeforeHandleShake;
      callback.on_handle_shake := handleShake;
      callback.on_connected := clientConnected;
      callback.on_recv := recvWebSocketMsg;
      callback.on_send := dxwslib.afterSendMsg;
      callback.on_closed := clientClosed;
      if FSSLCertFile <> '' then
      begin
        st := UTF8Encode(FSSLCertFile);
        tls.certFile.utf8data := @st[1];
        tls.certFile.len := Length(st);
      end
      else
      begin
        tls.certFile.utf8data := nil;
        tls.certFile.len := 0;
      end;

      if FSSLKeyFile <> '' then
      begin
        st := UTF8Encode(FSSLKeyFile);
        tls.keyFile.utf8data := @st[1];
        tls.keyFile.len := Length(st);
      end
      else
      begin
        tls.keyFile.utf8data := nil;
        tls.keyFile.len := 0;
      end;
      FServerHandle := websocket_listen(self.Port,callback,tls);
      if FServerHandle <> nil then
        FActive := True;
    end
    else
    begin
      //关闭服务
      if FServerHandle <> nil then
      begin
        stop_listen(FServerHandle);
        FServerHandle := nil;
      end;
      FActive := False;
    end;
  end;
end;

procedure TDxWebSocketServer.SetPort(const Value: Word);
begin
  if (FPort <> Value) and not FActive then
    FPort := Value;
end;

procedure TDxWebSocketServer.SetSSLCertFile(const Value: string);
begin
  if (FSSLCertFile <> Value) and not FActive then
    FSSLCertFile := Value;
end;

procedure TDxWebSocketServer.SetSSLKeyFile(const Value: string);
begin
  if (FSSLKeyFile <> Value) and not FActive then
    FSSLKeyFile := Value;
end;

{ TDxSocketClient }

procedure TDxSocketClient.AfterSendMsg(succeed: Boolean; msg: TWebSocketMsg);
begin
  if Assigned(FOnAfterSendMsg) then
    FOnAfterSendMsg(self,succeed,msg);
end;

procedure TDxSocketClient.Close;
var
  msg: TWebSocketMsg;
  writer: Pointer;
begin
  writer := AtomicCmpExchange(FWriter,nil,nil);
  if Writer <> nil then
  begin
    msg.msgType := Ord(Msg_Close);
    msg.data.utf8data := nil;
    msg.data.len := 0;
    send_data(msg,Writer);
    AtomicExchange(FWriter,nil)
  end;
end;

destructor TDxSocketClient.Destroy;
var
  writer: Pointer;
begin
  writer := AtomicCmpExchange(FWriter,nil,nil);
  if writer <> nil then
  begin
    free_writer(writer);
    AtomicExchange(FWriter,nil);
  end;
  inherited;
end;

procedure TDxSocketClient.DoClientClosed(closeCode: TCloseCode; code: Word;
  const reason: string);
begin
  if Assigned(FOnClosed) then
    FOnClosed(self,closeCode,code,reason);
end;

procedure TDxSocketClient.DoConnected;
begin

end;

procedure TDxSocketClient.DoRecvWebSocketMsg(msgType: TWebSocketMsgType;
  data: TStringData);
begin
   if Assigned(FOnRecvmsg) then
     FOnRecvmsg(self,TWebSocketMsgType(msgType),data)
end;

function TDxSocketClient.GetPeerAddr: string;
begin
  Result := FAddress.display;
end;

function TDxSocketClient.Ping(var buf: TRustBuffer): Boolean;
begin
  Result := Send(Msg_Ping,buf);
end;


function TDxSocketClient.Pong(var buf: TRustBuffer): Boolean;
begin
  Result := Send(Msg_Pong,buf);
end;


function TDxSocketClient.Send(tp: TWebSocketMsgType;buf: PByte; buflen: Integer): Boolean;
var
  msg: TWebSocketMsg;
  writer: Pointer;
begin
  writer := AtomicCmpExchange(FWriter,nil,nil);
  if writer <> nil then
  begin 
    msg.msgType := Ord(tp);
    msg.data.utf8data := buf;
    msg.data.len := buflen;
    result := send_data(msg,writer);
    if tp = Msg_Close then
      AtomicExchange(FWriter,nil);
  end
  else result := False;
end;

function TDxSocketClient.Send(tp: TWebSocketMsgType; var buf: TRustBuffer): Boolean;
begin
  if (buf.buf = nil) or (buf.buflen = 0) then
    Exit(False);
  result := Send(tp,buf.buf,buf.buflen); //发送到写完的
  buf.buf := nil;
  buf.buflen := 0;
  buf.Position := 0;
end;

function TDxSocketClient.SendText(value: UTF8String): Boolean;
var
  bf: TRustBuffer;
begin
  if value = '' then
    Exit(False);
  bf := TRustBuffer.Create(value);
  result := Send(Msg_Text,bf);
end;

function TDxSocketClient.Send(tp: TWebSocketMsgType;
  stream: TRustStream): Boolean;
begin
  if (stream.Memory = nil) or (stream.Size = 0) then
    Exit(False);
  result := Send(tp,stream.Memory,stream.Size); //发送到写完的
  stream.FCapacity := 0;
  TCustomMemoryStreamEx(Stream).FMemory := nil;
  TCustomMemoryStreamEx(Stream).FSize := 0;
  TCustomMemoryStreamEx(Stream).FPosition := 0;
end;

{ THandleShakeResponse }

procedure THandleShakeResponse.SetHeader(header, Value: UTF8String);
var
  k,v: TStringData;
begin
  if (resp <> nil) and (header <> '') then
  begin
    k.utf8data := @header[1];
    k.len := Length(header);
    v.len := Length(Value);
    if v.len = 0 then
      v.utf8data := nil
    else v.utf8data := @value[1];
    set_resp_header(resp,k,v);
  end;
end;

procedure THandleShakeResponse.SetStatusCode(code: Word);
begin
  if resp <> nil then
    set_resp_status(resp,code);
end;

{ TRustBuffer }

function TRustBuffer.AllocRustBuffer(l: Integer): Boolean;
begin
  self.buf := alloc(l);
  if self.buf <> nil then
    self.buflen := l
  else self.buflen := 0;
  Position := 0;
  Result := buf <> nil;
end;

class function TRustBuffer.Create(str: UTF8String): TRustBuffer;
begin
   Result.AllocRustBuffer(Length(str));
   Move(str[1],result.buf^,result.buflen);
   result.Position := result.buflen;
end;

function TRustBuffer.WriteString(st: UTF8String): Integer;
var
  left: Integer;
begin
  Result := Length(st);
  if buflen = 0 then
  begin
    if not self.AllocRustBuffer(Result) then
      Exit(0);
  end;
  left := buflen - Position;
  if Result > left then
  begin
    buflen := Position+result;
    Self.buf := realloc(buf,buflen,buflen);
  end;
  Inc(Position,Result);
  CopyMemory(Pointer(NativeUInt(buf)+Position),@st[1],Length(st));
end;

{ TRustStream }

const
  MemoryDelta = $2000; { Must be a power of 2 }

procedure TRustStream.Clear;
begin
  SetCapacity(0);
  TCustomMemoryStreamEx(self).FSize := 0;
  TCustomMemoryStreamEx(self).FPosition := 0;
end;

destructor TRustStream.Destroy;
begin
  Clear;
  inherited;
end;

procedure TRustStream.LoadFromFile(const FileName: string);
var
  Stream: TStream;
begin
  Stream := TFileStream.Create(FileName, fmOpenRead or fmShareDenyWrite);
  try
    LoadFromStream(Stream);
  finally
    Stream.Free;
  end;
end;

procedure TRustStream.LoadFromStream(Stream: TStream);
var
  Count: Int64;
begin
  Stream.Position := 0;
  Count := Stream.Size;
  SetSize(Count);
  if Count <> 0 then Stream.ReadBuffer(TCustomMemoryStreamEx(self).FMemory^, Count);
end;

function TRustStream.Realloc(var NewCapacity: Longint): Pointer;
var
  OldCap: Longint;
begin
  OldCap := FCapacity;
  if (NewCapacity > 0) and (NewCapacity <> TCustomMemoryStreamEx(self).FSize) then
    NewCapacity := (NewCapacity + (MemoryDelta - 1)) and not (MemoryDelta - 1);
  Result := Memory;
  if NewCapacity <> FCapacity then
  begin
    if NewCapacity = 0 then
    begin
      dealloc(Memory,OldCap);
      Result := nil;
    end
    else
    begin
      if Capacity = 0 then
        Result := alloc(NewCapacity)
      else
        result := dxwslib.realloc(Result,OldCap, NewCapacity);
      if Result = nil then raise EStreamError.CreateRes(@SMemoryStreamError);
    end;
  end;
end;

procedure TRustStream.SetCapacity(NewCapacity: NativeInt);
{$IF SizeOf(LongInt) = SizeOf(NativeInt)}
begin
  SetPointer(Realloc(LongInt(NewCapacity)), Size);
  FCapacity := NewCapacity;
end;
{$ELSEIF SizeOf(LongInt) < SizeOf(NativeInt)} // LP64 / Win64 platform
var
  NewCapacityLongInt: LongInt;
begin
  NewCapacityLongInt := NewCapacity;
  SetPointer(Realloc(NewCapacityLongInt), Size);
  NewCapacity := NewCapacityLongInt;
  FCapacity := NewCapacity;
end;
{$ENDIF}

procedure TRustStream.SetSize(const NewSize: Int64);
var
  OldPosition: NativeInt;
begin
  OldPosition := TCustomMemoryStreamEx(Self).FPosition;
  SetCapacity(NewSize);
  TCustomMemoryStreamEx(Self).FSize := NewSize;
  if OldPosition > NewSize then Seek(0, soEnd);
end;

procedure TRustStream.SetSize(NewSize: Longint);
begin
  SetSize(Int64(NewSize));
end;

function TRustStream.Write(const Buffer: TBytes; Offset,
  Count: Longint): Longint;
var
  Pos: Int64;
begin
  if (TCustomMemoryStreamEx(Self).FPosition >= 0) and (Count >= 0) then
  begin
    Pos := TCustomMemoryStreamEx(Self).FPosition + Count;
    if Pos > 0 then
    begin
      if Pos > TCustomMemoryStreamEx(Self).FSize then
      begin
        if Pos > FCapacity then
          SetCapacity(Pos);
        TCustomMemoryStreamEx(Self).FSize := Pos;
      end;
      System.Move(Buffer[Offset], (PByte(TCustomMemoryStreamEx(Self).FMemory) + TCustomMemoryStreamEx(Self).FPosition)^, Count);
      TCustomMemoryStreamEx(Self).FPosition := Pos;
      Result := Count;
      Exit;
    end;
  end;
  Result := 0;
end;

function TRustStream.Write(const Buffer; Count: Longint): Longint;
var
  Pos: Int64;
begin
  if (TCustomMemoryStreamEx(Self).FPosition >= 0) and (Count >= 0) then
  begin
    Pos := TCustomMemoryStreamEx(Self).FPosition + Count;
    if Pos > 0 then
    begin
      if Pos > TCustomMemoryStreamEx(Self).FSize then
      begin
        if Pos > FCapacity then
          SetCapacity(Pos);
        TCustomMemoryStreamEx(Self).FSize := Pos;
      end;
      System.Move(Buffer, (PByte(TCustomMemoryStreamEx(Self).FMemory) + TCustomMemoryStreamEx(Self).FPosition)^, Count);
      TCustomMemoryStreamEx(Self).FPosition := Pos;
      Result := Count;
      Exit;
    end;
  end;
  Result := 0;
end;



initialization
finalization
  finalize_ws_runtime
end.
