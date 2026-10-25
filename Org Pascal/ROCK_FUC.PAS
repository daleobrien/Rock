Unit Rock_fuc;
interface
uses Rock_var,WObjects, WinTypes, WinProcs, Strings, StdDlgs,WinCrt;

procedure PlaceObject(X,Y:integer);
Procedure Swap_Vertical(col,row:integer);
Procedure Swap_Diag(col,row:integer);
Procedure Delete_Bitmaps;
Function  Still(col,row:integer):Boolean;
Function  it_is_Empty_Below(col,row:integer):Boolean;
Function  a_clear_side(col,row:integer):Boolean;
Function  on_a_Diamond_or_Rock(col,row:integer):boolean;
Function  on_a_Diamond_or_Rock_or_Earth(col,row:integer):boolean;
Function  can_move_sideways(col,row:integer):Boolean;
Function  its_moving(col,row:integer):Boolean;
Function  Arrow_Key(Msg:TMessage):boolean;
Function  Shift:boolean;
Function  Space_clear(col,row:integer):Boolean; {excludcing Diamonds}
Procedure Display_Score(Inc:integer);
Function  Eatible_Diamond(col,row:integer):Boolean;

Procedure  Load_BitMaps;
Procedure  Init_Screen(Index:Integer);
Procedure  Change_to_Falling(Col,Row:integer);
Procedure  Change_to_Rest(Col,Row:integer);
Procedure  Lifemove;
Procedure  Explode(index:integer);
Procedure  Check_if_man_still_happening;

procedure  Drop_all;
Procedure  Flash_Exit;
procedure  morphchange;
Function   In_exit:boolean;
Procedure  Next_Screen;

implementation

procedure PlaceObject(X,Y:integer);
 Var 
  MemObj:THandle;Memdc:HDC;
 begin
  Memdc:=CreateCompatibleDC(DC);
  MemObj:=SelectObject(MemDC,Map[X,Y]);
  BitBlt(DC,(X-1)*20,(Y-1)*20,20,20,MemDC,0,0,SrcCopy);
  DeleteObject(MemObj);
  DeleteDc(MemDC);
 end;

Function Still(col,row:integer):Boolean;
 begin
  if (map[col,row]=Boulder) or (map[col,row]=FBoulder) or
	  (map[col,row]=Diamond) or (map[col,row]=FDiamond) then
	    Still:=True
  else Still:=False;
 end;

Function it_is_Empty_Below(col,row:integer):Boolean;
  begin
	if map[col,row+1]=Empty then
	 it_is_Empty_Below:=True else
	 it_is_Empty_Below:=False;
  end;

Function a_clear_side(col,row:integer):Boolean;
 begin
   if ( (map[col-1,row+1]=Empty) and (Map[col-1,row]=Empty) ) or
	  ( (map[col+1,row+1]=Empty) and (Map[col+1,row]=Empty) ) then
     a_clear_Side:=True else a_clear_side:=False;
 end;

Function on_a_Diamond_or_Rock(col,row:integer):boolean;
 begin
	if ((map[col,row+1]=Boulder ) or (map[col,row+1]=Diamond )  or
		 (map[col,row+1]=FBoulder) or (map[col,row+1]=FDiamond)  or
       (map[col,row+1]=FBoulder) or (map[col,row+1]=FDiamond)  or
		 (map[col,row+1]=Bounds  ) or (map[col,row+1]=Wall) ) then
  on_a_Diamond_or_Rock:=True else  on_a_Diamond_or_Rock:=false;
 end;

Function on_a_Diamond_or_Rock_or_Earth(col,row:integer):boolean;
 begin
  if on_a_Diamond_or_Rock(col,row) or (map[col,row+1]=earth) then
   on_a_Diamond_or_Rock_or_Earth:=true else
   on_a_Diamond_or_Rock_or_Earth:=False;
 end;

Function can_move_sideways(col,row:integer):Boolean;
 begin
  if on_a_Diamond_or_Rock(col,row) and a_clear_side(col,row) then
	can_move_sideways:=true else can_move_sideways:=False;
 end;

Function its_moving(col,row:integer):Boolean;
 begin
  if (map[col,row]=FnBoulder) or (map[col,row]=FnDiamond) then
	its_moving:=true else its_moving:=false;
 end;

Procedure Swap_Vertical(col,row:integer);
 begin
  Map[col,row+1]:=Map[col,row];
  Map[col,row]:=Empty;
  PlaceObject(col,row);
  Change_to_Falling(col,row+1);
  PlaceObject(col,row+1);
 end;

Procedure Swap_Diag(col,row:integer);
 var Side:integer;
 begin
  if Random(1) < 0.5 then Side:=1 else Side:=-1;
  if (map[col+Side,row+1]=empty) and (map[col+Side,row]=empty) then
   begin
	 map[col+Side,row]:=map[col,row];
  	 Change_to_Falling(col+Side,row);
    PlaceObject(col+Side,row);
	end else
	begin
	 map[col-Side,row]:=map[col,row];
	 Change_to_Falling(col-Side,row);
	 PlaceObject(col-Side,row);
	end;

	map[col,row]:=Empty;
   PlaceObject(col,row);
 end;

Procedure Delete_Bitmaps;
var i:integer;
 begin
  DeleteDc(DC);
  DeleteObject(Boulder);
  DeleteObject(FBoulder);
  DeleteObject(FNBoulder);
  DeleteObject(Diamond);
  DeleteObject(FDiamond);
  DeleteObject(FnDiamond);
  DeleteObject(Empty);
  DeleteObject(Wall);
  DeleteObject(Earth);
  DeleteObject(Man1);
  DeleteObject(Man2);
  DeleteObject(Bounds);

  DeleteObject(MorphS[0]);
  DeleteObject(MorphS[1]);
  DeleteObject(MorphS[2]);
  DeleteObject(MorphS[3]);
  DeleteObject(Morph);


  DeleteObject(Score1);
  DeleteObject(Score2);
  DeleteObject(Door);
  for i:=0 to 9 do
	DeleteObject(pic[i]);
  
 end;

Function Arrow_Key(Msg:TMessage):boolean;
 begin
  if (Msg.Wparam = 37) or (Msg.Wparam = 38) or (Msg.Wparam = 39) or
	  (Msg.Wparam = 40) then Arrow_Key:=True else Arrow_Key:=False;
 end;

Function Shift:boolean;
 begin
  if  GetAsyncKeyState(VK_Shift) < 0 then Shift:=True else Shift:=False;
 end;

Function Space_clear(col,row:integer):Boolean; 
 begin
  if (Map[col,row]=Earth    ) or (Map[col,row]=Empty   ) or
	  (Map[col,row]=FDiamond ) or (Map[col,row]=Diamond ) or
	  (Map[col,row]=FnDiamond) or (Map[col,row]=Door    ) then
	 Space_clear:=True else Space_clear:=false;
 end;

Procedure Display_Score(Inc:integer);
 var  temp:integer;
	MemoryDC:HDC;
	MemoryObj:THandle;
 begin
  if Sound then MessageBeep(0);
  Diamonds:=Diamonds+Inc;
  if Diamonds > no_of_required_Diamonds then
   Diamonds:= no_of_required_Diamonds;
  if Diamonds=no_of_required_Diamonds then
	begin
	 Map[exit.x,exit.y]:=Door;
	 PlaceObject(exit.x,exit.y);
	 Exit_Availible:=True;
	end;

  temp:=Round(Diamonds*196/no_of_required_Diamonds);

  MemoryDC:=CreateCompatibleDC(DC);
  MemoryObj:=SelectObject(MemoryDC,Score2);
  StretchBlt(DC,103+temp,0,197-temp,20,MemoryDC,0,0,2,20,SrcCopy);
  DeleteDC(MemoryDC);
  DeleteObject(MemoryObj);

  MemoryDC:=CreateCompatibleDC(DC);
  MemoryObj:=SelectObject(MemoryDC,Score1);
  StretchBlt(DC,103,     0,    temp,20,MemoryDC,0,0,2,20,SrcCopy);
  DeleteDC(MemoryDC);
  DeleteObject(MemoryObj);
 end;

Function Eatible_Diamond(col,row:integer):Boolean;
 begin
  if (map[col,row]=Diamond) or (map[col,row]=FDiamond) or
  (map[col,row]=FnDiamond) then
   Eatible_Diamond:=True else Eatible_Diamond:=False;
 end;

Procedure Set_Square(a,b,width,depth:integer;Name:THandle);
 var i,j,w,x,y,z:integer;
 begin
  if a=1 then w:=1 else w:=0;
  if (a+Width) > max_col then x:=a+width-max_col+1 else x:=1;

  if b=1 then y:=1 else y:=0;
  if (b+Depth) > max_row then z:=b+depth-max_row+1 else z:=1;

  for i:=0+w to width-x do
	for j:=0+y to Depth-z do
	 begin
	  if (Map[a+i,b+j]=Man1) or (Map[a+i,b+j]=Man2) then
		begin
		 Just_Died:=true;
		  Alive:=False;
      end;
	  Map[a+i,b+j]:=Name;
	  PlaceObject(a+i,b+j);
    end;

 end;

Function Boxed_in(col,row:integer):Boolean;
 begin
if (Map[col-1,row] <> Empty) and (Map[col+1,row] <> Empty)
  and (Map[col,row-1] <> Empty) and (Map[col,row+1] <> Empty) then
	boxed_in:=True else Boxed_in:=False;
 end;

Function Check_if_Fng_object_above(i:integer):boolean;
 begin
  if ((map[life[i].X,life[i].Y-1]=FnDiamond) or
		(map[life[i].X,life[i].Y-1]=FnBoulder)) then
	  Check_if_Fng_object_above:=true else
	  Check_if_Fng_object_above:=False;
 end;

Procedure Explode(index:integer);
var row,col:integer;
 begin
  col:=life[index].X;row:=life[index].Y;             
  if (Check_if_Fng_object_above(index)) and Not(Life[index].dead) then
  begin
	 Set_Square(col-1,row-1,3,3,FDiamond);
	 Life[index].dead:=true;
	end;
  if Boxed_in(col,row) and Not Life[index].dead then
	begin
	 Life[index].length:=Life[index].length-1;
	 if life[index].length=0 then
     begin
	  Set_Square(col-1,row-1,3,3,FDiamond);
	  Life[index].dead:=true;
    end;
   end;
 end;

Function Life_there(col,row:integer):boolean;
 var t:integer;
 begin
  life_there:=false;
  for t:=0 to 9 do
   if map[col,row]=pic[t] then life_there:=true;
 end;

Function Empty_or_Man_or_life(col,row:integer):Boolean;
 begin
  if (Map[col,row]=Man1) or (Map[col,row]=Man2) or (Map[col,row]=Empty)
	  {or life_there(col,row)}  then
	Empty_or_man_or_life:=True else Empty_or_Man_or_life:=False;
 end;

Procedure Lifemove;
 var j,k,m,index:integer;
	  dx,dy,rad:integer;
begin
 for index:=1 to no_of_life do
 With life[index] do
  begin
  Explode(index);

 if Not Dead then
  begin {begin if not dead}
	j:=0;
	 k:=Index mod 2;
	 if ((1+index) mod 4)>1 then m:=0 else m:=1;
    
  repeat 
	 side:=side+1-2*k;

	 if side < 1 then side:=side+4;
	 if side > 4 then side:=side-4;

	 dx:=Delta_State[side].x;
	 dy:=Delta_State[side].y;
 	 if Empty_or_Man_or_life(x+dx,y+dy) then
	  begin {Can move}
		if (Map[x+dx,y+dy]=Man1) or (Map[x+dx,y+dy]=Man2) then
       begin 
		  Alive:=False;  Just_died:=true; Map[x+dx,y+dy]:=Empty;
		 end;
 	   Map[x+dx,y+dy]:=Map[x,y];Map[x,y]:=Empty;
		oX:=x; oY:=y; x:=x+dx; y:=y+dy;
       {-1 = straight ahead, -2 RH, or +1 straight, -2 LH}
		 side:=side-2+m*(1+k*2);
		 if side < 1 then side:=side+4;
		j:=3;
	  end;
	 j:=j+1;
  until j>3;
  if  State > 8 then State:=0 else State:=State+1;
  Map[x,y]:=Pic[state];
  PlaceObject(oX,oY);
  PlaceObject(x,y);
end; {end if not dead }
 end;
end;

Procedure  Change_to_Rest(Col,Row:integer);
begin
 if map[col,row]=FnBoulder then map[col,row]:=FBoulder;
 if map[col,row]=FnDiamond then map[col,row]:=FDiamond;
 if map[col,row]=Boulder then map[col,row]:=FBoulder;
 if map[col,row]=Diamond then map[col,row]:=FDiamond;
end;

Procedure  Change_to_Falling(Col,Row:integer);
begin
 if map[col,row]=Boulder  then map[col,row]:=FnBoulder;
 if map[col,row]=Diamond  then map[col,row]:=FnDiamond;
 if map[col,row]=FBoulder then map[col,row]:=FnBoulder;
 if map[col,row]=FDiamond then map[col,row]:=FnDiamond;
end;
                   
Procedure Load_BitMaps;

 begin
  Boulder:=  loadBitmap(HInstance,'Boulder');
  Diamond:=  loadBitmap(HInstance,'Diamond');   
  FBoulder:= loadBitmap(HInstance,'FBoulder');  
  FDiamond:= loadBitmap(HInstance,'FDiamond');
  FnBoulder:=LoadBitmap(Hinstance,'FnBoulder');
  FnDiamond:=LoadBitmap(Hinstance,'FnDiamond');

  MorphS[0]:=    LoadBitmap(Hinstance,'Morph1');
  MorphS[1]:=    LoadBitmap(Hinstance,'Morph2');
  MorphS[2]:=    LoadBitmap(Hinstance,'Morph3');
  MorphS[3]:=    LoadBitmap(Hinstance,'Morph4');

  Morph:=MorphS[0];

  Wall:=     loadBitmap(HInstance,'Wall');     
  Earth:=    loadBitmap(HInstance,'Earth');    
  Empty:=    loadBitmap(HInstance,'Empty');    
  Man1:=		 loadBitmap(HInstance,'Man1');
  Man2:=		 loadBitmap(HInstance,'Man2');
  Bounds:=	 loadBitmap(HInstance,'Bounds');
  Score1:=   loadBitmap(Hinstance,'Score1');
  Score2:=   loadBitmap(Hinstance,'Score2');
  Door:=     LoadBitmap(Hinstance,'Door');

  Pic[0]:=   LoadBitmap(Hinstance,'Life0');
  Pic[1]:=   LoadBitmap(Hinstance,'Life1');
  Pic[2]:=   LoadBitmap(Hinstance,'Life2');
  Pic[3]:=   LoadBitmap(Hinstance,'Life3');
  Pic[4]:=   LoadBitmap(Hinstance,'Life4');
  Pic[5]:=   LoadBitmap(Hinstance,'Life5');
  Pic[6]:=   LoadBitmap(Hinstance,'Life6');
  Pic[7]:=   LoadBitmap(Hinstance,'Life7');
  Pic[8]:=   LoadBitmap(Hinstance,'Life8');
  Pic[9]:=   LoadBitmap(Hinstance,'Life9');
 end;

Procedure Set_bounds;
 var i:integer;
 begin
  for i:=1 to Max_row do
   begin
    Map[1,i]:=Bounds;Map[Max_col,i]:=Bounds;
   end;
  for i:=1 to Max_col do
	begin
	 Map[i,1]:=Bounds;
    Map[i,Max_row]:=Bounds;
	end;
 end;

Procedure Set_Wall(a,b,length:integer;Horizontal:boolean);
 var i:integer;
 begin
  if Horizontal then
  for i:=0 to length-1 do
   begin 
 	 Map[a+i,b]:=Wall;
	end else
  for i:=0 to length-1 do
   begin 
	 Map[a,b+i]:=Wall;
	end;
 end;


Procedure Set_All_Earth;
  var row,col:integer;
 begin
 for row:=1 to max_row do
    for col:=1 to max_col do
	  Map[col,row]:=Earth;
 end;

Procedure Set_morph(col,row,index:integer);
 begin
  Map[col,row]:=Morph;
  Morphic[index].X:=col;
  Morphic[index].Y:=row;
  Morphic[index].state:=0; {0..3}
 end;

Procedure Set_life(i,a,b:integer);
 begin
	With life[i] do
	 begin
	  X:=a;
	  Y:=b;
	  Side:=1;
	  State:=i mod 9;
	  Map[a,b]:=pic[i mod 9];
	  Dead:=False;
	  length:=30;
	 end
 end;

Procedure Set_Man(a,b:integer);
 begin
  Map[a,b]:=Man1;
  Man_Position.X:=a;
  Man_Position.Y:=b;
 end;

Procedure  Init_Screen(Index:Integer);
 var row,col,i:integer;
 begin
  no_of_life:=0;
  morph_no:=0;
  Just_died:=False;
  Exit_Availible:=False;
  Alive:=True;
  Sound:=False;
  RandSeed:=$ffff;
  Diamonds:=0;
  Exit_Availible:=False;
  Set_all_Earth;
  last_level:=9;

 if (Index=1) or (Index>last_level) or (Index<1) then {No life just get all rocks} 
  begin
	No_of_life:=0;
	Set_all_Earth;	no_of_required_Diamonds:=67;

	Exit.X:=Max_col;Exit.Y:=19;
	Map[Max_col,18]:=Boulder;

	for i:= 1 to 100 do Map[Random(Max_Col-1)+1,Random(Max_row-1)+1]:=Diamond;
   for i:= 1 to 70  do Map[Random(Max_Col-1)+1,Random(Max_row-1)+1]:=Boulder;

   
	Set_Wall(8,2,18,false);
	Set_Wall(15,2,18,false);
	Set_Wall(23,2,18,false);
	
	Set_Square(8,9,1,4,Earth);	Set_Square(15,9,1,4,Earth);
	Set_Square(23,9,1,4,Earth);
	Set_man(2,2);
   end;{index 1}


 if (Index=2)  then {}
  begin
	Exit_Availible:=False;	Set_all_Earth;
	no_of_required_Diamonds:=35;
	Exit.X:=Max_col; Exit.Y:=19;
	Map[Max_col,18]:=Boulder;
	for i:= 1 to 50 do Map[Random(Max_Col-1)+1,Random(Max_row-1)+1]:=Diamond;
   for i:= 1 to 30  do Map[Random(Max_Col-1)+1,Random(Max_row-1)+1]:=Boulder;
	No_of_Life:=3; 
	Set_Wall(8,2,18,false);
	Set_Wall(15,2,18,false);
	Set_Wall(23,2,18,false);
	Set_life(1,10,9);           Set_life(2,14,9);       Set_life(3,20,9);
	Set_Square(8,9,16,4,Empty);
	Set_man(2,2);
  end;{index 3}

   if Index=3 then
	begin
	 no_of_required_Diamonds:=50;
	 Exit.X:=Max_col;
    Exit.Y:=19;
	 Map[Max_col,18]:=boulder;

	 Set_all_Earth;	 Set_Man(2,9);	 No_of_life:=8;

	 Set_life(1,5,3); Set_life(3,10,3);
	 Set_life(5,5,8); Set_life(7,10,8);
	 Set_life(2,5,13);Set_life(4,10,13);
	 Set_life(6,5,18);Set_life(8,10,18);

	 Set_Square(4,2,8,19,Empty);
	 Set_Square(5,3,6,16,Earth); 
	 Set_Square(6,4,4,14,Diamond);
  	 Set_Square(13,3,17,16,Boulder);
	end;


 if (Index=4) then
  begin
	no_of_required_Diamonds:=40;
	Exit.X:=Max_col;   Exit.Y:=19;
	Map[Max_col,18]:=boulder;
  	No_of_Life:=14; {Must be correct}

	Set_Square(9,10,15,1,Boulder);
	for i:=1 to 7 do
	 begin
	  Set_life(i*2-1,2*i+8,6);  Map[2*i+8,7]:=Empty;
	  Set_life(i*2,  2*i+8,13); Map[2*i+8,14]:=Empty;
	 end;
   Set_man(2,10);
  end;


 if Index=5 then
  begin
	no_of_required_Diamonds:=20;
	Exit.X:=15;  Exit.Y:=10;
	Map[Max_col,18]:=boulder;
	Set_Square(12,2,1,1,Boulder);Set_Square(14,2,1,1,Boulder);
	Set_Square(16,2,1,1,Boulder);Set_Square(18,2,1,1,Boulder);
	Set_Square(12,7,7,7,Empty);
	no_of_life:=4;
	Set_life(1,12,7);	Set_life(2,18,7);	Set_life(3,12,13);Set_life(4,18,13);
	Map[Exit.X,Exit.Y]:=Bounds;
   Set_man(2,10);
  end;

 if Index=6 then
  begin
	no_of_life:=0;
	no_of_required_Diamonds:=50;
	Exit.X:=Max_col;Exit.Y:=max_row-3;
	Map[Max_col,Max_row-4]:=boulder;
	Set_Square(5,4,10,5,Boulder);

   morph_no:=1;
   Set_Morph(15,14,0);

	Set_Man(Exit.X-1,Exit.Y);
   end;

 if Index=7 then
  begin
	no_of_required_Diamonds:=50;
	Exit.X:=Max_col;Exit.Y:=max_row-3;
	Map[Max_col,Max_row-4]:=boulder;
	Set_Square(2,2,30,20,Empty);
	no_of_life:=30;
	for i:=1 to 10 do
	 begin
	  Set_Square(2+i*2,4,1,1,Diamond);
	  Set_Square(2+i*2,5,1,1,Earth);
     Set_life(i*3-1,2+i*2,6);

	  Set_Square(  2+i*2,10,1,1,Diamond);
	  Set_Square(  2+i*2,11,1,1,Earth);
	  Set_life(i*3-2,2+i*2,12);

	  Set_Square(  2+i*2,16,1,1,Diamond);
	  Set_Square(  2+i*2,17,1,1,Earth);
	  Set_life(i*3,2+i*2,18);

    end;
  
	Set_Man(Exit.X-1,Exit.Y);

  end;
 if Index=8 then
  begin
	no_of_required_Diamonds:=50;
	Exit.X:=Max_col;Exit.Y:=10;
	Map[Max_col,9]:=boulder;
	Set_Square(2,2,30,20,Empty);
	no_of_life:=50;
	for i:=1 to 10 do
	 begin
	  Set_Square(2+i*2,4,1,1,Boulder);	  Set_Square(2+i*2,5,1,1,Earth);
     Set_life(i*5-4,2+i*2,6);
	  Set_Square(    2+i*2,7,1,1,Boulder);Set_Square(    2+i*2,8,1,1,Earth);
     Set_life(i*5-3,2+i*2,9);
	  Set_Square(  2+i*2,10,1,1,Boulder); Set_Square(  2+i*2,11,1,1,Earth);
	  Set_life(i*5-2,2+i*2,12);
	  Set_Square(  2+i*2,13,1,1,Boulder); Set_Square(  2+i*2,14,1,1,Earth);
	  Set_life(i*5-1,2+i*2,15);
	  Set_Square(  2+i*2,16,1,1,Boulder); Set_Square(  2+i*2,17,1,1,Earth);
	  Set_life(i*5,2+i*2,18);
    end;
  
	Set_Man(Exit.X-1,10);
  end;

   if Index=9 then
  begin
	no_of_required_Diamonds:=2;

	Exit.X:=Max_col;
	Exit.Y:=10;
	
	no_of_life:=12;

	Set_Square(11,5,5,1,Empty); Set_life(11,15,5);Set_life(12,21, 5);
	Set_Square(11,7,5,1,Empty); Set_life(1,11,7); Set_life(6, 21, 7);
	Set_Square(11,9,5,1,Empty); Set_life(2,12,9); Set_life(7, 20, 9);
	Set_Square(11,11,5,1,Empty);Set_life(3,13,11);Set_life(8, 19,11);
	Set_Square(11,13,5,1,Empty);Set_life(4,14,13);Set_life(9, 18,13);
	Set_Square(11,15,5,1,Empty);Set_life(5,15,15);Set_life(10,17,15);
   Set_Square(17,2,5,20,Empty);
	Set_Wall(16,2,18,false);	Set_Wall(17,10,1,false);
	Set_Square(16,10,1,1,Diamond);
	Set_Square(Max_col-1,9,1,1,Boulder);
	Set_man(2,10);
  end;

  Set_Bounds;
 end;{Init_Screen}

 Procedure Next_Screen;
 var s,t:integer;
 begin
  Screen_no:=Screen_no+1;
  if Screen_no=last_level+1 then
   begin
	 MessageBox(Hwin,'All done','You have escaped Death',mb_ok);
    Screen_no:=1;
   end;
  Init_Screen(Screen_no);

  for s:=1 to Max_col do
	for t:=1 to Max_row do
	 PlaceObject(s,t);

  Display_Score(0);
  end;

 Function Check_if_Fng_object_above_man:boolean;
 begin
  with Man_position do
  if ((map[X,Y-1]=FnDiamond) or (map[X,Y-1]=FnBoulder)) and Alive then
	  Check_if_Fng_object_above_man:=true else
	  Check_if_Fng_object_above_man:=False;
 end;

Procedure Check_if_man_still_happening;
 begin
  with Man_position do
  begin
  if Alive then
  begin 
	if Map[X,Y]=Man1 then Map[x,y]:=Man2 else Map[x,y]:=Man1;
	PlaceObject(X,Y);
  end;

  if Check_if_Fng_object_above_man then
  begin
	just_died:=true;
	Alive:=False;
  end;

  if Just_died then
   begin
	 Just_died:=False;
	 Map[x,y]:=Map[x,y-1];	 Map[x,y-1]:=Empty;
	 PlaceObject(x,y-1);  	 PlaceObject(x,y);
	 MessageBox(HWin,'You are mine !','Death says ..',mb_ok);
	 if Sound then MessageBeep(0);
	 Screen_no:=Screen_no-1;
	 if Screen_no<0 then Screen_no:=0;
	 Alive:=True;
	 Next_Screen;
	end;
  end;
 end;

 function Is_Rock(col,row:integer):boolean;
  begin
	if (Map[col,row]=Boulder) or (Map[col,row]=FBoulder) or(Map[col,row]=FNBoulder) then
    Is_Rock:=True else Is_Rock:=False;
  end;

 Procedure Rock_on_Morph(col,row:integer);
  begin
	if (row < (max_row-1)) and (row > 2) then
    begin
	  if (Map[col,row]=Morph)
  	      and Is_Rock(col,row-1) then
		begin
		 Map[col,row-1]:=Empty;    PlaceObject(col,row-1);
    
		 Map[col,row+1]:=FDiamond; PlaceObject(col,row+1);
		end;
     if (Map[col,row]=Morph) and Eatible_Diamond(col,row-1) then
	   begin
		 Map[col,row-1]:=Empty;    PlaceObject(col,row-1);
		 Map[col,row+1]:=FBoulder; PlaceObject(col,row+1);
		end;
    end;
  end;

 Procedure Drop_all;
  var col,row:integer;   
 Begin
  for col:=1 to max_col do
   for row:=max_row-1 downto 1 do
    begin
 	  if Still(col,row) then
	   begin
	    if it_is_Empty_Below(col,row) then  Swap_vertical(col,row) else
 		 if can_move_sideways(col,row) then  Swap_Diag(col,row);
      end;

	  if its_moving(col,row) and on_a_Diamond_or_Rock_or_Earth(col,row) then
	   begin
		 Change_to_rest(col,row);
		 PlaceObject(col,row);
	   end
	  else if its_moving(col,row) then
     begin
      if it_is_Empty_Below(col,row) then Swap_vertical(col,row)
      else if can_move_sideways(col,row) then	Swap_Diag(col,row);
     end;
     Rock_On_Morph(col,row);
	end;
  End;


Procedure Flash_Exit;
var Rect:TRect;
 begin
	 Rect.Top:=Exit.Y*20-20;
	 Rect.bottom:=Exit.Y*20;
	 Rect.Left:=Exit.X*20-20;
	 Rect.Right:=Exit.X*20;
	 InvertRect(DC,Rect);
{  if Exit_Availible then
   begin
	 Rect.top:=4;
	 Rect.Bottom:=8;
	 Rect.Left:=104;
	 Rect.Right:=104+196;
	 InvertRect(DC,Rect);
	 Rect.top:=12;
	 Rect.Bottom:=16;
    InvertRect(DC,Rect);
	end;}
 end;

procedure morphchange;
 var rr:integer;
 begin
 { for rr:=1 to morph_no do}
	begin
	 Morph:=MorphS[random(3)];
	 Map[Morphic[0].X,Morphic[0].Y]:=Morph;
	 PlaceObject(Morphic[0].X,Morphic[0].Y);
   end;
 end;

Function   In_exit:boolean;
 begin
  if Exit_Availible and (Man_position.X=Exit.X) and
  (Man_position.Y=Exit.Y) then
	In_Exit:=True Else In_Exit:=False;
 end;

begin
end.