{module, 'Elixir.Lambda'}.  %% version = 0

{exports, [{'__info__',1},
           {captures,0},
           {module_info,0},
           {module_info,1},
           {start,0}]}.

{attributes, []}.

{labels, 19}.


{function, '__info__', 1, 2}.
  {label,1}.
    {line,[]}.
    {func_info,{atom,'Elixir.Lambda'},{atom,'__info__'},1}.
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
    {move,{atom,'Elixir.Lambda'},{x,0}}.
    {line,[]}.
    {call_ext_only,2,{extfunc,erlang,get_module_info,2}}.
  {label,4}.
    {move,{literal,[{captures,0},{start,0}]},{x,0}}.
    return.
  {label,5}.
    {move,{atom,'Elixir.Lambda'},{x,0}}.
    return.
  {label,6}.
    {move,nil,{x,0}}.
    return.


{function, captures, 0, 8}.
  {label,7}.
    {line,[{location,"lambda.ex",8}]}.
    {func_info,{atom,'Elixir.Lambda'},{atom,captures},0}.
  {label,8}.
    {make_fun2,{f,18},0,0,0}.
    return.


{function, start, 0, 10}.
  {label,9}.
    {line,[{location,"lambda.ex",3}]}.
    {func_info,{atom,'Elixir.Lambda'},{atom,start},0}.
  {label,10}.
    {allocate,0,0}.
    {make_fun2,{f,16},0,0,0}.
    {move,{x,0},{x,1}}.
    {move,{integer,1},{x,0}}.
    {line,[{location,"lambda.ex",5}]}.
    {call_fun,1}.
    {deallocate,0}.
    return.


{function, module_info, 0, 12}.
  {label,11}.
    {line,[]}.
    {func_info,{atom,'Elixir.Lambda'},{atom,module_info},0}.
  {label,12}.
    {move,{atom,'Elixir.Lambda'},{x,0}}.
    {line,[]}.
    {call_ext_only,1,{extfunc,erlang,get_module_info,1}}.


{function, module_info, 1, 14}.
  {label,13}.
    {line,[]}.
    {func_info,{atom,'Elixir.Lambda'},{atom,module_info},1}.
  {label,14}.
    {move,{x,0},{x,1}}.
    {move,{atom,'Elixir.Lambda'},{x,0}}.
    {line,[]}.
    {call_ext_only,2,{extfunc,erlang,get_module_info,2}}.


{function, '-start/0-fun-0-', 1, 16}.
  {label,15}.
    {line,[{location,"lambda.ex",4}]}.
    {func_info,{atom,'Elixir.Lambda'},{atom,'-start/0-fun-0-'},1}.
  {label,16}.
    {line,[{location,"lambda.ex",4}]}.
    {gc_bif,'+',{f,0},1,[{x,0},{integer,1}],{x,0}}.
    return.


{function, '-captures/0-fun-0-', 1, 18}.
  {label,17}.
    {line,[{location,"lambda.ex",10}]}.
    {func_info,{atom,'Elixir.Lambda'},{atom,'-captures/0-fun-0-'},1}.
  {label,18}.
    {line,[{location,"lambda.ex",10}]}.
    {gc_bif,'-',{f,0},1,[{x,0},{integer,2}],{x,0}}.
    return.
