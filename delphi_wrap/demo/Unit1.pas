unit Unit1;

interface

uses
  Winapi.Windows, Winapi.Messages, System.SysUtils, System.Variants, System.Classes, Vcl.Graphics,
  Vcl.Controls, Vcl.Forms, Vcl.Dialogs, Vcl.StdCtrls,dxwslib,Winapi.Winsock2,
  Vcl.ExtCtrls;

type
  TForm1 = class(TForm)
    Button1: TButton;
    Panel1: TPanel;
    Memo1: TMemo;
    ListBox1: TListBox;
    Edit1: TEdit;
    Button2: TButton;
    procedure FormCreate(Sender: TObject);
    procedure Button1Click(Sender: TObject);
    procedure Button2Click(Sender: TObject);
  private
    { Private declarations }
    procedure DoBeforeHandleShake(sender: TObject;peerAddr: PSocketAddrInfo;var succed: Boolean);
    procedure DoHandleShake(sender: TObject;request: THandleShakeRequest;resp: THandleShakeResponse;var succed: Boolean);
    procedure DoClientConnected(Client: TDxSocketClient);
    procedure DoClientClosed(Client: TDxSocketClient;closeCode: TCloseCode;code: Word;reason: string);
    procedure DoRecvMsg(sender: TObject;Client: TDxSocketClient; msgType: TWebSocketMsgType;data: TStringData);
  public
    { Public declarations }
    server: TDxWebSocketServer;
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
begin
  if ListBox1.ItemIndex = -1 then
  begin
    ShowMessage('��ѡ��Ҫ���͵Ŀͻ���');
    Exit;
  end;
  client := ListBox1.Items.Objects[ListBox1.ItemIndex] as TDxSocketClient;
  client.SendText(Edit1.Text);
end;

procedure TForm1.DoBeforeHandleShake(sender: TObject; peerAddr: PSocketAddrInfo;
  var succed: Boolean);
begin
  if GetCurrentThreadId = MainThreadID then
    Memo1.Lines.Add(peerAddr^.display + ' ����׼������')
  else TThread.Synchronize(nil,procedure
    begin
      Memo1.Lines.Add(peerAddr^.display + ' ����׼������')
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
    Memo1.Lines.Add(Client.PeerAddr + ' �ر��ˣ��ر�ԭ��'+reason);
    idx := ListBox1.Items.IndexOfObject(Client);
    if idx <> -1 then
      ListBox1.Items.Delete(idx);
  end
  else TThread.Synchronize(nil,procedure
    begin
      Memo1.Lines.Add(Client.PeerAddr + ' �ر��ˣ��ر�ԭ��'+reason);
      idx := ListBox1.Items.IndexOfObject(Client);
      if idx <> -1 then
        ListBox1.Items.Delete(idx);
    end);
  Client.Free;
end;

procedure TForm1.DoClientConnected(Client: TDxSocketClient);
var
  ip: string;
begin
  Client.SendText('���������������͵�text��Ϣ');
  ip := Client.PeerAddr;
  if GetCurrentThreadId = MainThreadID then
  begin
    ListBox1.Items.AddObject(ip,Client);
    Memo1.Lines.Add(ip + ' ����������')
  end
  else TThread.Synchronize(nil,procedure
    begin
      ListBox1.Items.AddObject(ip,Client);
      Memo1.Lines.Add(ip + ' ����������')
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
    if request.Headers['token'] = '123' then
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
        Memo1.Lines.Add(FormatDateTime('HH:NN:SS.zzz',Now)+'��'+Client.PeerAddr+'����');
        Memo1.Lines.Add(data.Value)
      end
      else
      begin
        TThread.Synchronize(nil,procedure
          begin
            Memo1.Lines.Add(FormatDateTime('HH:NN:SS.zzz',Now)+'��'+Client.PeerAddr+'����');
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
  server.BeforeHandleShake := DoBeforeHandleShake;
  server.OnHandleShake := DoHandleShake;
  server.OnClientConnected := DoClientConnected;
  server.OnClientClosed := DoClientClosed;
  server.OnRecvMsg := DoRecvMsg;
end;

end.