unit Unit1;

interface

uses
  Winapi.Windows, Winapi.Messages, System.SysUtils, System.Variants, System.Classes, Vcl.Graphics,
  Vcl.Controls, Vcl.Forms, Vcl.Dialogs, Vcl.StdCtrls,dxwslib,Winapi.Winsock2,System.Rtti,
  Vcl.ExtCtrls;

type
  TForm1 = class(TForm)
    Button1: TButton;
    Panel1: TPanel;
    Memo1: TMemo;
    ListBox1: TListBox;
    Edit1: TEdit;
    Button2: TButton;
    Button3: TButton;
    procedure FormCreate(Sender: TObject);
    procedure Button1Click(Sender: TObject);
    procedure Button2Click(Sender: TObject);
    procedure Button3Click(Sender: TObject);
  private
    { Private declarations }
    procedure DoBeforeHandleShake(sender: TObject;peerAddr: PSocketAddrInfo;var succed: Boolean);
    procedure DoHandleShake(sender: TObject;request: THandleShakeRequest;resp: THandleShakeResponse;var succed: Boolean);
    procedure DoClientConnected(Client: TDxSocketClient);
    procedure DoClientClosed(Client: TDxSocketClient;closeCode: TCloseCode;code: Word;reason: string);
    procedure DoRecvMsg(sender: TObject;Client: TDxSocketClient; msgType: TWebSocketMsgType;data: TStringData);
    procedure DoAfterSend(client: TDxSocketClient;succeed: Boolean; msg: TWebSocketMsg);

    procedure DoClientConnected1(sender: TObject);
  public
    { Public declarations }
    server: TDxWebSocketServer;
    tmpStream: TRustStream;
    client: TDxWebSocketClient;
  end;

var
  Form1: TForm1;

implementation

{$R *.dfm}

procedure TForm1.Button1Click(Sender: TObject);
begin
  server.Active := True;
end;

procedure TForm1.Button2Click(Sender: TObject);
var
  Client: TDxSocketClient;
  buf: TRustBuffer;
  st: UTF8String;
  i: Integer;
begin
  if ListBox1.ItemIndex = -1 then
  begin
    ShowMessage('请选择要发送的客户端');
    Exit;
  end;
  client := ListBox1.Items.Objects[ListBox1.ItemIndex] as TDxSocketClient;
  //client.SendText(Edit1.Text);
  for i := 0 to 10 do
  begin
    Client.SendText('测试不得闲发斯蒂芬234123412431324')
  end;
end;

procedure TForm1.Button3Click(Sender: TObject);
begin
  if Client = nil then
  begin
    client := TDxWebSocketClient.Create;
    client.AddProto('chat3');
    client.AddProto('file4');
    client.AddHead('user','dxsoft');
    Client.AddHead('token','123');
    client.OnConnected := DoClientConnected1;
    client.Connect('ws://127.0.0.1:8088/test1');
  end;
end;

procedure TForm1.DoAfterSend(client: TDxSocketClient; succeed: Boolean;
  msg: TWebSocketMsg);
begin
  //不需要释放
  if (msg.data.utf8data <> nil) and (msg.data.len <> 0) then
  begin
    if succeed then
      TThread.Synchronize(nil,procedure
        begin
          Memo1.Lines.Add('发送成功：'+msg.data.Value)
        end)
    else TThread.Synchronize(nil,procedure
        begin
          Memo1.Lines.Add('发送失败：'+msg.data.Value)
        end)
  end;
end;

procedure TForm1.DoBeforeHandleShake(sender: TObject; peerAddr: PSocketAddrInfo;
  var succed: Boolean);
begin
  if GetCurrentThreadId = MainThreadID then
    Memo1.Lines.Add(peerAddr^.display + ' 正在准备连接')
  else TThread.Synchronize(nil,procedure
    begin
      Memo1.Lines.Add(peerAddr^.display + ' 正在准备连接')
    end);
  succed := True;
end;

procedure TForm1.DoClientClosed(Client: TDxSocketClient; closeCode: TCloseCode;
  code: Word; reason: string);
var
  idx: Integer;
begin
  if GetCurrentThreadId = MainThreadID then
  begin
    Memo1.Lines.Add(Client.PeerAddr + ' 关闭了，关闭原因：'+reason);
    idx := ListBox1.Items.IndexOfObject(Client);
    if idx <> -1 then
      ListBox1.Items.Delete(idx);
  end
  else TThread.Synchronize(nil,procedure
    begin
      Memo1.Lines.Add(Client.PeerAddr + ' 关闭了，关闭原因：'+reason);
      idx := ListBox1.Items.IndexOfObject(Client);
      if idx <> -1 then
        ListBox1.Items.Delete(idx);
    end);
  Client.Free;
end;

procedure TForm1.DoClientConnected1(sender: TObject);
begin
  if GetCurrentThreadId = MainThreadID then
  begin
  end
  else TThread.Synchronize(nil,procedure
    begin
      Showmessage('客户端连接成功');
    end);
end;

procedure TForm1.DoClientConnected(Client: TDxSocketClient);
var
  ip: string;
begin
  Client.SendText('测试连接上来发送的text信息');
  ip := Client.PeerAddr;
  if GetCurrentThreadId = MainThreadID then
  begin
    ListBox1.Items.AddObject(ip,Client);
    Memo1.Lines.Add(ip + ' 连接上来了')
  end
  else TThread.Synchronize(nil,procedure
    begin
      ListBox1.Items.AddObject(ip,Client);
      Memo1.Lines.Add(ip + ' 连接上来了')
    end);
end;

procedure TForm1.DoHandleShake(sender: TObject; request: THandleShakeRequest;
  resp: THandleShakeResponse; var succed: Boolean);
begin
  succed := False;
  if request.RequestPath = '/test' then
  begin
    succed := True;
  end
  else if request.RequestPath = '/test1' then
  begin
    if (request.Headers['token'] = '123') and (request.Headers['user'] = 'dxsoft') then
      Succed := True;
  end;
  if not succed then
  begin
    resp.SetStatusCode(406);
    resp.SetHeader('auth','requested');
  end;
end;

procedure TForm1.DoRecvMsg(sender: TObject; Client: TDxSocketClient;
  msgType: TWebSocketMsgType; data: TStringData);
begin
  case msgType of
  Msg_Text:
    begin
      if GetCurrentThreadId = MainThreadID then
      begin
        Memo1.Lines.Add(FormatDateTime('HH:NN:SS.zzz',Now)+'【'+Client.PeerAddr+'】：');
        Memo1.Lines.Add(data.Value)
      end
      else
      begin
        TThread.Synchronize(nil,procedure
          begin
            Memo1.Lines.Add(FormatDateTime('HH:NN:SS.zzz',Now)+'【'+Client.PeerAddr+'】：');
            Memo1.Lines.Add(data.Value)
          end);
      end;
    end;
  Msg_Bin: ;
  Msg_Ping: ;
  Msg_Pong: ;
  Msg_Close: ;
  Msg_Frame: ;
  Msg_None: ;
  end;
end;

procedure TForm1.FormCreate(Sender: TObject);
begin


  server := TDxWebSocketServer.Create;
  server.Port := 8088;
  server.OnAfterSendMsg := DoAfterSend;
  server.BeforeHandleShake := DoBeforeHandleShake;
  server.OnHandleShake := DoHandleShake;
  server.OnClientConnected := DoClientConnected;
  server.OnClientClosed := DoClientClosed;
  server.OnRecvMsg := DoRecvMsg;


end;

end.
