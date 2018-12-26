{module, 'Elixir.Bin'}.  %% version = 0

{exports, [{'__info__',1},
           {add,1},
           {interpol,1},
           {module_info,0},
           {module_info,1},
           {start,0}]}.

{attributes, []}.

{labels, 19}.


{function, '__info__', 1, 2}.
  {label,1}.
    {line,[]}.
    {func_info,{atom,'Elixir.Bin'},{atom,'__info__'},1}.
  {label,2}.
    {test,is_atom,{f,1},[{x,0}]}.
    {select_val,{x,0},
                {f,1},
                {list,[{atom,compile},
                       {f,3},
                       {atom,md5},
                       {f,3},
                       {atom,attributes},
                       {f,3},
                       {atom,functions},
                       {f,4},
                       {atom,module},
                       {f,5},
                       {atom,macros},
                       {f,6},
                       {atom,deprecated},
                       {f,6}]}}.
  {label,3}.
    {move,{x,0},{x,1}}.
    {move,{atom,'Elixir.Bin'},{x,0}}.
    {line,[]}.
    {call_ext_only,2,{extfunc,erlang,get_module_info,2}}.
  {label,4}.
    {move,{literal,[{add,1},{interpol,1},{start,0}]},{x,0}}.
    return.
  {label,5}.
    {move,{atom,'Elixir.Bin'},{x,0}}.
    return.
  {label,6}.
    {move,nil,{x,0}}.
    return.


{function, add, 1, 8}.
  {label,7}.
    {line,[{location,"binary.ex",7}]}.
    {func_info,{atom,'Elixir.Bin'},{atom,add},1}.
  {label,8}.
    {line,[{location,"binary.ex",8}]}.
    {bs_append,{f,0},{integer,8},0,0,8,{x,0},{field_flags,[]},{x,1}}.
    {bs_put_string,1,{string,"1"}}.
    {move,{x,1},{x,0}}.
    return.


{function, interpol, 1, 10}.
  {label,9}.
    {line,[{location,"binary.ex",11}]}.
    {func_info,{atom,'Elixir.Bin'},{atom,interpol},1}.
  {label,10}.
    {allocate,0,1}.
    {test,is_binary,{f,11},[{x,0}]}.
    {jump,{f,12}}.
  {label,11}.
    {line,[{location,"binary.ex",12}]}.
    {call_ext,1,{extfunc,'Elixir.String.Chars',to_string,1}}.
  {label,12}.
    {line,[{location,"binary.ex",12}]}.
    {gc_bif,byte_size,{f,0},1,[{x,0}],{x,1}}.
    {bs_add,{f,0},[{x,1},{integer,6},1],{x,1}}.
    {bs_init2,{f,0},{x,1},0,1,{field_flags,[]},{x,1}}.
    {bs_put_string,4,{string,"asd "}}.
    {bs_put_binary,{f,0},{atom,all},8,{field_flags,[unsigned,big]},{x,0}}.
    {bs_put_string,2,{string," q"}}.
    {move,{x,1},{x,0}}.
    {deallocate,0}.
    return.


{function, start, 0, 14}.
  {label,13}.
    {line,[{location,"binary.ex",3}]}.
    {func_info,{atom,'Elixir.Bin'},{atom,start},0}.
  {label,14}.
    {move,{literal,<<"2">>},{x,0}}.
    {call_only,1,{f,8}}.


{function, module_info, 0, 16}.
  {label,15}.
    {line,[]}.
    {func_info,{atom,'Elixir.Bin'},{atom,module_info},0}.
  {label,16}.
    {move,{atom,'Elixir.Bin'},{x,0}}.
    {line,[]}.
    {call_ext_only,1,{extfunc,erlang,get_module_info,1}}.


{function, module_info, 1, 18}.
  {label,17}.
    {line,[]}.
    {func_info,{atom,'Elixir.Bin'},{atom,module_info},1}.
  {label,18}.
    {move,{x,0},{x,1}}.
    {move,{atom,'Elixir.Bin'},{x,0}}.
    {line,[]}.
    {call_ext_only,2,{extfunc,erlang,get_module_info,2}}.
