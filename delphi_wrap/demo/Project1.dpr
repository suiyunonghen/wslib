program Project1;

uses
  Winapi.Windows,
  Vcl.Forms,
  Unit1 in 'Unit1.pas' {Form1},
  dxwslib in '..\dxwslib.pas';

{$R *.res}

procedure log_msg(level: Integer;logData: TStringData);stdcall;
begin
  OutputDebugString(PChar(logData.Value));
end;

begin
  Application.Initialize;
  Application.MainFormOnTaskbar := True;
  Application.CreateForm(TForm1, Form1);
  InitWslib(log_msg,0,false,'wslib_x86.dll');
  //InitWslib(log_msg,0,False,'E:\delphi\programs\myprograms\myOpenSource\websocket\wslib\target\i686-pc-windows-msvc\release\wslib.dll');
  //Application.Run;
end.
