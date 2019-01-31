{module, 'Elixir.Bin'}.  %% version = 0

{exports, [{'__info__',1},
           {add,2},
           {interpol,1},
           {interpol2,2},
           {interpol_types,3},
           {module_info,0},
           {module_info,1},
           {parse,1},
           {start,0},
           {str,0}]}.

{attributes, []}.

{labels, 36}.


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
    {move,{literal,[{add,2},
                    {interpol,1},
                    {interpol2,2},
                    {interpol_types,3},
                    {parse,1},
                    {start,0},
                    {str,0}]},
          {x,0}}.
    return.
  {label,5}.
    {move,{atom,'Elixir.Bin'},{x,0}}.
    return.
  {label,6}.
    {move,nil,{x,0}}.
    return.


{function, add, 2, 8}.
  {label,7}.
    {line,[{location,"binary.ex",24}]}.
    {func_info,{atom,'Elixir.Bin'},{atom,add},2}.
  {label,8}.
    {line,[{location,"binary.ex",25}]}.
    {bs_append,{f,0},{integer,8},0,2,8,{x,0},{field_flags,[]},{x,2}}.
    {bs_put_string,1,{string,"1"}}.
    {line,[{location,"binary.ex",26}]}.
    {gc_bif,'+',{f,0},3,[{x,1},{integer,2}],{x,0}}.
    {line,[{location,"binary.ex",27}]}.
    {bs_append,{f,0},{integer,8},0,0,8,{x,2},{field_flags,[]},{x,0}}.
    {bs_put_string,1,{string,"3"}}.
    return.


{function, interpol, 1, 10}.
  {label,9}.
    {line,[{location,"binary.ex",30}]}.
    {func_info,{atom,'Elixir.Bin'},{atom,interpol},1}.
  {label,10}.
    {allocate,0,1}.
    {test,is_binary,{f,11},[{x,0}]}.
    {jump,{f,12}}.
  {label,11}.
    {line,[{location,"binary.ex",31}]}.
    {call_ext,1,{extfunc,'Elixir.String.Chars',to_string,1}}.
  {label,12}.
    {line,[{location,"binary.ex",31}]}.
    {gc_bif,byte_size,{f,0},1,[{x,0}],{x,1}}.
    {bs_add,{f,0},[{x,1},{integer,6},1],{x,1}}.
    {bs_init2,{f,0},{x,1},0,1,{field_flags,[]},{x,1}}.
    {bs_put_string,4,{string,"asd "}}.
    {bs_put_binary,{f,0},{atom,all},8,{field_flags,[unsigned,big]},{x,0}}.
    {bs_put_string,2,{string," q"}}.
    {move,{x,1},{x,0}}.
    {deallocate,0}.
    return.


{function, interpol2, 2, 14}.
  {label,13}.
    {line,[{location,"binary.ex",38}]}.
    {func_info,{atom,'Elixir.Bin'},{atom,interpol2},2}.
  {label,14}.
    {allocate,1,2}.
    {move,{x,1},{y,0}}.
    {test,is_binary,{f,15},[{x,0}]}.
    {jump,{f,16}}.
  {label,15}.
    {line,[{location,"binary.ex",39}]}.
    {call_ext,1,{extfunc,'Elixir.String.Chars',to_string,1}}.
  {label,16}.
    {line,[{location,"binary.ex",39}]}.
    {gc_bif,'+',{f,0},1,[{y,0},{integer,2}],{x,1}}.
    {move,{x,0},{y,0}}.
    {test,is_binary,{f,17},[{x,1}]}.
    {move,{x,1},{x,0}}.
    {jump,{f,18}}.
  {label,17}.
    {move,{x,1},{x,0}}.
    {line,[{location,"binary.ex",39}]}.
    {call_ext,1,{extfunc,'Elixir.String.Chars',to_string,1}}.
  {label,18}.
    {line,[{location,"binary.ex",39}]}.
    {gc_bif,byte_size,{f,0},1,[{x,0}],{x,1}}.
    {gc_bif,byte_size,{f,0},2,[{y,0}],{x,2}}.
    {bs_add,{f,0},[{x,1},{x,2},1],{x,1}}.
    {bs_add,{f,0},[{x,1},{integer,8},1],{x,1}}.
    {bs_init2,{f,0},{x,1},0,1,{field_flags,[]},{x,1}}.
    {bs_put_string,4,{string,"asd "}}.
    {bs_put_binary,{f,0},{atom,all},8,{field_flags,[unsigned,big]},{y,0}}.
    {bs_put_string,4,{string," q1 "}}.
    {bs_put_binary,{f,0},{atom,all},8,{field_flags,[unsigned,big]},{x,0}}.
    {move,{x,1},{x,0}}.
    {deallocate,1}.
    return.


{function, interpol_types, 3, 20}.
  {label,19}.
    {line,[{location,"binary.ex",34}]}.
    {func_info,{atom,'Elixir.Bin'},{atom,interpol_types},3}.
  {label,20}.
    {allocate,2,3}.
    {move,{x,2},{y,0}}.
    {move,{x,1},{y,1}}.
    {test,is_binary,{f,21},[{x,0}]}.
    {jump,{f,22}}.
  {label,21}.
    {line,[{location,"binary.ex",35}]}.
    {call_ext,1,{extfunc,'Elixir.String.Chars',to_string,1}}.
  {label,22}.
    {line,[{location,"binary.ex",35}]}.
    {gc_bif,byte_size,{f,0},1,[{x,0}],{x,1}}.
    {bs_add,{f,0},[{x,1},{integer,13},1],{x,1}}.
    {bs_init2,{f,0},{x,1},0,1,{field_flags,[]},{x,1}}.
    {bs_put_string,4,{string,"asd "}}.
    {bs_put_binary,{f,0},{atom,all},8,{field_flags,[unsigned,big]},{x,0}}.
    {bs_put_integer,{f,0},{integer,8},1,{field_flags,[unsigned,big]},{y,1}}.
    {bs_put_float,{f,0},{integer,64},1,{field_flags,[unsigned,big]},{y,0}}.
    {move,{x,1},{x,0}}.
    {deallocate,2}.
    return.


{function, parse, 1, 24}.
  {label,23}.
    {line,[{location,"binary.ex",12}]}.
    {func_info,{atom,'Elixir.Bin'},{atom,parse},1}.
  {label,24}.
    {test,bs_start_match2,{f,23},1,[{x,0},0],{x,0}}.
    {test,bs_get_integer2,
          {f,26},
          1,
          [{x,0},
           {integer,8},
           1,
           {field_flags,[{anno,[12,{file,"binary.ex"}]},unsigned,big]}],
          {x,1}}.
    {test,is_eq_exact,{f,25},[{x,1},{integer,104}]}.
    {test,bs_match_string,{f,25},[{x,0},40,{string,"ello "}]}.
    {test,bs_get_binary2,
          {f,25},
          2,
          [{x,0},
           {atom,all},
           8,
           {field_flags,[{anno,[12,{file,"binary.ex"}]},unsigned,big]}],
          {x,0}}.
    {'%',{no_bin_opt,{binary_used_in,return},[12,{file,"binary.ex"}]}}.
    return.
  {label,25}.
    {bs_restore2,{x,0},{atom,start}}.
  {label,26}.
    {test,bs_get_float2,
          {f,27},
          1,
          [{x,0},
           {integer,64},
           1,
           {field_flags,[{anno,[16,{file,"binary.ex"}]},unsigned,big]}],
          {x,1}}.
    {test,bs_test_tail2,{f,27},[{x,0},0]}.
    {move,{x,1},{x,0}}.
    return.
  {label,27}.
    {bs_context_to_binary,{x,0}}.
    {jump,{f,23}}.


{function, start, 0, 29}.
  {label,28}.
    {line,[{location,"binary.ex",3}]}.
    {func_info,{atom,'Elixir.Bin'},{atom,start},0}.
  {label,29}.
    {allocate,0,0}.
    {move,{literal,<<63,244,0,0,0,0,0,0>>},{x,0}}.
    {line,[{location,"binary.ex",7}]}.
    {call,1,{f,24}}.
    {move,{literal,<<"hello world">>},{x,0}}.
    {call_last,1,{f,24},0}.


{function, str, 0, 31}.
  {label,30}.
    {line,[{location,"binary.ex",20}]}.
    {func_info,{atom,'Elixir.Bin'},{atom,str},0}.
  {label,31}.
    {move,{literal,<<"34">>},{x,0}}.
    return.


{function, module_info, 0, 33}.
  {label,32}.
    {line,[]}.
    {func_info,{atom,'Elixir.Bin'},{atom,module_info},0}.
  {label,33}.
    {move,{atom,'Elixir.Bin'},{x,0}}.
    {line,[]}.
    {call_ext_only,1,{extfunc,erlang,get_module_info,1}}.


{function, module_info, 1, 35}.
  {label,34}.
    {line,[]}.
    {func_info,{atom,'Elixir.Bin'},{atom,module_info},1}.
  {label,35}.
    {move,{x,0},{x,1}}.
    {move,{atom,'Elixir.Bin'},{x,0}}.
    {line,[]}.
    {call_ext_only,2,{extfunc,erlang,get_module_info,2}}.
