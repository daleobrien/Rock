Unit Rock_Var;
interface
uses WObjects, WinTypes, WinProcs, Strings, StdDlgs;
const
	Max_x=600; 
	Max_y=400;
	Max_row=20;
	Max_col=31;
	Delta_State:  array[1..4] of Tpoint=((X:-1;Y:0),(X:0;Y:-1),(X:1;Y: 0),(X:0;Y:1));

Type
 TPic		=	array[0..9] of Hbitmap;
 Tmorph	=	array[0..3] of Hbitmap;
Var
  Boulder,FBoulder,FnBoulder, Diamond,FDiamond,FnDiamond,
  Man1,Man2,Empty,Earth,Wall,Bounds,Score1,Score2,Door,morph,

  life0,life1,life2,life3,life4,life5,life6,life7,life8,life9:HBitmap;
  morph1,morph2,morph3,morph4:HBitmap;

  Alive,Sound, Just_Died,Exit_Availible:Boolean;

  No_of_Life,no_of_required_Diamonds:integer;
  last_level:integer;
  Exit:TPoint;
  Once_DC:Boolean;
  DC:HDC;
  Hwin:Thandle;
  Screen_no:integer;
  Old:TPoint;
  Delta:Tpoint;

  Pic		:TPic;
  morphs	:Tmorph;

  Man_Position :record
   x,y,ox,oy,length:integer; end;

  Morph_no:integer;

  Morphic:array[0..100] of record
	X:integer;
	Y:integer;
	State:integer;
  end;

  Life:array[1..100] of Record
     X	:integer;
	  Y	:Integer;
	  Side:integer;
	  oX	,
	  oY	:integer;
	  dead:boolean;
	  State:integer;
	  length:integer;
  end;

	Diamonds:integer;

 type
	screen = array [1..Max_col,1..Max_row] of THandle;

 var
	Map:Screen;

implementation



begin end.