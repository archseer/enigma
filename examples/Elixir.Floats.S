{module, 'Elixir.Floats'}.  %% version = 0

{exports, [{'__info__',1},{add,1},{module_info,0},{module_info,1},{start,0}]}.

{attributes, []}.

{labels, 15}.


{function, '__info__', 1, 2}.
  {label,1}.
    {line,[]}.
    {func_info,{atom,'Elixir.Floats'},{atom,'__info__'},1}.
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
    {move,{atom,'Elixir.Floats'},{x,0}}.
    {line,[]}.
    {call_ext_only,2,{extfunc,erlang,get_module_info,2}}.
  {label,4}.
    {move,{literal,[{add,1},{start,0}]},{x,0}}.
    return.
  {label,5}.
    {move,{atom,'Elixir.Floats'},{x,0}}.
    return.
  {label,6}.
    {move,nil,{x,0}}.
    return.


{function, add, 1, 8}.
  {label,7}.
    {line,[{location,"floats.ex",6}]}.
    {func_info,{atom,'Elixir.Floats'},{atom,add},1}.
  {label,8}.
    {test_heap,{alloc,[{words,0},{floats,1}]},1}.
    {line,[{location,"floats.ex",7}]}.
    {fmove,{float,1.25},{fr,0}}.
    {fconv,{x,0},{fr,1}}.
    fclearerror.
    {bif,fadd,{f,0},[{fr,0},{fr,1}],{fr,1}}. # => remap to op/
    {line,[{location,"floats.ex",7}]}.
    {fmove,{float,2.0},{fr,2}}.
    {bif,fdiv,{f,0},[{fr,1},{fr,2}],{fr,1}}. #remap to op?
    {fcheckerror,{f,0}}.
    {fmove,{fr,1},{x,0}}.
    return.


{function, start, 0, 10}.
  {label,9}.
    {line,[{location,"floats.ex",2}]}.
    {func_info,{atom,'Elixir.Floats'},{atom,start},0}.
  {label,10}.
    {move,{float,1.5},{x,0}}.
    {call_only,1,{f,8}}.


{function, module_info, 0, 12}.
  {label,11}.
    {line,[]}.
    {func_info,{atom,'Elixir.Floats'},{atom,module_info},0}.
  {label,12}.
    {move,{atom,'Elixir.Floats'},{x,0}}.
    {line,[]}.
    {call_ext_only,1,{extfunc,erlang,get_module_info,1}}.


{function, module_info, 1, 14}.
  {label,13}.
    {line,[]}.
    {func_info,{atom,'Elixir.Floats'},{atom,module_info},1}.
  {label,14}.
    {move,{x,0},{x,1}}.
    {move,{atom,'Elixir.Floats'},{x,0}}.
    {line,[]}.
    {call_ext_only,2,{extfunc,erlang,get_module_info,2}}.
