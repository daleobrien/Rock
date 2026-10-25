{$D '* Dale O'Brien, Beta Verison of Rock, 1993, Jan , Coded using Borland TPW 1.5 * }
Program rock;

{$R rock20.res}
uses WObjects, WinTypes, WinProcs, Strings, StdDlgs, WinCrt,Rock_Fuc,Rock_var;

 Type
  TRockApp = object(TApplication)
    procedure InitMainWindow; virtual;
   end;

  PRock = ^TRock;
  TRock	   = object(TWindow)
    constructor Init(ATitle: PChar);
	 Destructor  Done;virtual;
	 Procedure   SetupWindow;virtual;

{	 Procedure   GetWindowClass(var AwndClass:TWndClass); virtual;}
{	Procedure WMVScroll(var Msg:Tmessage); virtual wm_first + wm_Vscroll;}

	 procedure   Paint(PaintDC: HDC; var PaintInfo: TPaintStruct); Virtual;

	 procedure   WMLButtonDown(Var Msg: TMessage); virtual wm_First + wm_LButtonDown;
	 Procedure   Keyboard(Var Msg: TMessage); virtual wm_First + wm_KeyDown;
	 Procedure   Timer(var Msg:Tmessage);virtual wm_first + wm_timer;
	 Procedure   About(Var Msg:TMessage);virtual cm_first + 100;
	 Procedure   Level(Var Msg:TMessage);virtual cm_first + 200;
  end;

Procedure   TRock.About(Var Msg:TMessage);
 begin
  MessageBox(Hwindow,'1993, Dale ','Beta Verison 1.0 - Rock',mb_ok);
 end;

Procedure  TRock.Level(Var Msg:TMessage);
 var Text:pchar;
 begin
  Text:='1';
  Application^.ExecDialog(New(PInputDialog,Init(@self,'You Choose',
  'Enter 1..9',Text,Sizeof(text))));
  Screen_no:=Ord(Text[0])-49;
  Next_Screen;
 end;

procedure TRock.Paint(PaintDC: HDC; var PaintInfo: TPaintStruct);
 var
  row,col:integer;
  a,b,c,d:integer;
 begin
  if Not Once_DC then
	begin
	 Once_DC:=True;
	 DC:=GetDC(HWindow);
	end;
  b:=(PaintInfo.rcpaint.top) div 20;
  a:=(PaintInfo.rcpaint.left) div 20;
  d:=(PaintInfo.rcpaint.bottom+20) div 20;
  c:=(PaintInfo.rcpaint.right+20) div 20;

  if c>max_col then c:=max_col;
  if d>max_row then d:=max_row;
{  Writeln('paint area=(',a,',',b,'),(',c,',',d,')');}


  for row:=b to d do
   for col:=a to c do
    PlaceObject(col,row);
  Display_Score(0);
 end;

{ Procedure TRock.WMVScroll(var Msg:Tmessage);
  begin

  end;}

Procedure TRock.WMLButtonDown(Var Msg: TMessage);
 var x,y,row,col:integer;
 begin
  x:=Msg.lparamlo;
  y:=Msg.LparamHi;
  col:=Round((x+9.5)/20);
  row:=Round((y+9.5)/20);
 end;

Destructor  TRock.Done;
 begin
  TWindow.Done;
  Delete_Bitmaps;
 end;

constructor TRock.Init(ATitle: PChar);
 begin
  TWindow.Init(nil, ATitle);
  Attr.X:=0;Attr.Y:=0; Attr.H:=Max_y+40; Attr.W:=Max_x+20;
  Attr.Style:=ws_DlgFrame  or ws_SysMenu;
  Attr.Menu:=LoadMenu(HInstance,'Menu_1');
  Load_BitMaps;
  Init_Screen(1);  {set back to 1}
  Hwin:=Hwindow;
 end;

Procedure TRock.SetupWindow;
 begin
   TWindow.SetupWindow;
	SetTimer(Hwindow,0,200,nil);
 end;


Procedure TRock.Keyboard(Var Msg: TMessage);
 var dx,dy:integer;
 begin
{ if Msg.WParam=vk_return then Next_Screen(2); }
With Man_Position do
 begin
  if Arrow_Key(Msg) and Alive then
   begin
	 ox:=x;oy:=y;
	 dx:=Delta_STATE[Msg.Wparam-36].x;
	 dy:=Delta_STATE[Msg.Wparam-36].y;
	 x:=dx+x;	 y:=dy+y;
 {Push Boulder}
  if ( dx <> 0) and (( Map[x,y]=FBoulder) or (Map[x,y]=Boulder )) and
	  ( Map[dx+x,y]=Empty) then
 	begin
	 Map[x,y]:=Map[ox,y]; Map[ox,y]:=Empty;    Map[dx+x,y]:=FBoulder;

    if Shift then
     begin
      Map[ox,y]:=Map[x,y];	Map[x,y]:=Empty;
		x:=ox;
	  end;
	 PlaceObject(ox,y);
	 PlaceObject(ox+dx,oy+dy);
	 PlaceObject(ox+2*dx,oy+dy);
	 ox:=x;
   end;

 {Dig or Dig and Move }
  if Space_Clear(x,y) then
	begin
	 if Eatible_Diamond(x,y) then Display_Score(1);
 	 Map[x,y]:=Map[ox,oy];
	 Map[ox,oy]:=Empty;
	 if Shift then
	 begin
	  Map[ox,oy]:=Map[x,y];
	  Map[x,y]:=Empty;
	  x:=ox;y:=oy;
    end;
    PlaceObject(ox,oy);
	 PlaceObject(ox+dx,oy+dy);
	end else
   begin	x:=ox;y:=oy;	 end;

end;{Check if a arrow key was pressed}

  Check_if_man_still_happening;
  if In_exit then Next_Screen;
end;

 end;
 

 Procedure TRock.Timer(var Msg:Tmessage);
  var
   i:integer;
  begin
	Drop_all;  {Check if life below as well}
	Check_if_man_still_happening;
	Flash_Exit;
	Lifemove;
	morphchange;
  end;

 Procedure TRockApp.InitMainWindow;
  begin
	MainWindow := New(PRock, Init('Rock'));
  end;

var
  RockApp: TRockApp;
  count:integer ;
begin
  Diamonds:=0;
  Screen_no:=1;

  Once_dc:=False;
  RockApp.Init('Rocky');
  RockApp.Run;
  RockApp.Done;
end.
