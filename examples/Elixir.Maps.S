{module, 'Elixir.Maps'}.  %% version = 0

{exports, [{'__info__',1},
           {m,0},
           {match,1},
           {module_info,0},
           {module_info,1},
           {start,0}]}.

{attributes, []}.

{labels, 18}.


{function, '__info__', 1, 2}.
  {label,1}.
    {line,[]}.
    {func_info,{atom,'Elixir.Maps'},{atom,'__info__'},1}.
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
    {move,{atom,'Elixir.Maps'},{x,0}}.
    {line,[]}.
    {call_ext_only,2,{extfunc,erlang,get_module_info,2}}.
  {label,4}.
    {move,{literal,[{m,0},{match,1},{start,0}]},{x,0}}.
    return.
  {label,5}.
    {move,{atom,'Elixir.Maps'},{x,0}}.
    return.
  {label,6}.
    {move,nil,{x,0}}.
    return.


{function, m, 0, 8}.
  {label,7}.
    {line,[{location,"map.ex",12}]}.
    {func_info,{atom,'Elixir.Maps'},{atom,m},0}.
  {label,8}.
    {move,{literal,#{a => 1}},{x,0}}.
    return.


{function, match, 1, 10}.
  {label,9}.
    {line,[{location,"map.ex",15}]}.
    {func_info,{atom,'Elixir.Maps'},{atom,match},1}.
  {label,10}.
    {test,is_map,{f,9},[{x,0}]}.
    {get_map_elements,{f,9},{x,0},{list,[{atom,a},{x,2},{atom,b},{x,1}]}}.
    {line,[{location,"map.ex",16}]}.
    {gc_bif,'+',{f,0},3,[{x,2},{x,1}],{x,0}}.
    return.


{function, start, 0, 12}.
  {label,11}.
    {line,[{location,"map.ex",3}]}.
    {func_info,{atom,'Elixir.Maps'},{atom,start},0}.
  {label,12}.
    {allocate,0,0}.
    {line,[{location,"map.ex",7}]}.
    {call,0,{f,8}}.
    {test,is_map,{f,13},[{x,0}]}.
    {line,[{location,"map.ex",8}]}.
    {put_map_exact,{f,0},{x,0},{x,0},1,{list,[{atom,b},{integer,1}]}}.
    {call_last,1,{f,10},0}.
  {label,13}.
    {test_heap,3,1}.
    {put_tuple,2,{x,1}}.
    {put,{atom,badmap}}.
    {put,{x,0}}.
    {move,{x,1},{x,0}}.
    {line,[{location,"map.ex",8}]}.
    {call_ext,1,{extfunc,erlang,error,1}}.


{function, module_info, 0, 15}.
  {label,14}.
    {line,[]}.
    {func_info,{atom,'Elixir.Maps'},{atom,module_info},0}.
  {label,15}.
    {move,{atom,'Elixir.Maps'},{x,0}}.
    {line,[]}.
    {call_ext_only,1,{extfunc,erlang,get_module_info,1}}.


{function, module_info, 1, 17}.
  {label,16}.
    {line,[]}.
    {func_info,{atom,'Elixir.Maps'},{atom,module_info},1}.
  {label,17}.
    {move,{x,0},{x,1}}.
    {move,{atom,'Elixir.Maps'},{x,0}}.
    {line,[]}.
    {call_ext_only,2,{extfunc,erlang,get_module_info,2}}.
