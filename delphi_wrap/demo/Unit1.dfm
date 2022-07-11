object Form1: TForm1
  Left = 0
  Top = 0
  Caption = 'Form1'
  ClientHeight = 511
  ClientWidth = 821
  Color = clBtnFace
  Font.Charset = DEFAULT_CHARSET
  Font.Color = clWindowText
  Font.Height = -11
  Font.Name = 'Tahoma'
  Font.Style = []
  OldCreateOrder = False
  OnCreate = FormCreate
  PixelsPerInch = 96
  TextHeight = 13
  object Button1: TButton
    Left = 8
    Top = 9
    Width = 89
    Height = 33
    Caption = #30417#21548
    TabOrder = 0
    OnClick = Button1Click
  end
  object Panel1: TPanel
    Left = 0
    Top = 47
    Width = 821
    Height = 464
    Align = alBottom
    Anchors = [akLeft, akTop, akRight, akBottom]
    Caption = 'Panel1'
    TabOrder = 1
    object Memo1: TMemo
      Left = 1
      Top = 1
      Width = 698
      Height = 462
      Align = alClient
      TabOrder = 0
    end
    object ListBox1: TListBox
      Left = 699
      Top = 1
      Width = 121
      Height = 462
      Align = alRight
      ItemHeight = 13
      TabOrder = 1
    end
  end
  object Edit1: TEdit
    Left = 264
    Top = 16
    Width = 401
    Height = 21
    TabOrder = 2
    Text = 'Edit1'
  end
  object Button2: TButton
    Left = 671
    Top = 17
    Width = 75
    Height = 25
    Caption = #21457#36865
    TabOrder = 3
    OnClick = Button2Click
  end
  object Button3: TButton
    Left = 168
    Top = 16
    Width = 75
    Height = 25
    Caption = #23458#25143#31471#36830#25509
    TabOrder = 4
    OnClick = Button3Click
  end
end
