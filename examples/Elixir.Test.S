{module, 'Elixir.Test'}.  %% version = 0

{exports, [{'__info__',1},{hello,1},{module_info,0},{module_info,1},{start,0}]}.

{attributes, []}.

{labels, 15}.


{function, '__info__', 1, 2}.
  {label,1}.
    {line,[]}.
    {func_info,{atom,'Elixir.Test'},{atom,'__info__'},1}.
  {label,2}.
    {test,is_atom,{f,1},[{x,0}]}.
    {select_val,{x,0},
                {f,1},
                ---> breaks around here
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
    {move,{atom,'Elixir.Test'},{x,0}}.
    {line,[]}.
    {call_ext_only,2,{extfunc,erlang,get_module_info,2}}.
  {label,4}.
    {move,{literal,[{hello,1},{start,0}]},{x,0}}.
    return.
  {label,5}.
    {move,{atom,'Elixir.Test'},{x,0}}.
    return.
  {label,6}.
    {move,nil,{x,0}}.
    return.


{function, hello, 1, 8}.
  {label,7}.
    {line,[{location,"spawn.ex",7}]}.
    {func_info,{atom,'Elixir.Test'},{atom,hello},1}.
  {label,8}.
    return.


{function, start, 0, 10}.
  {label,9}.
    {line,[{location,"spawn.ex",3}]}.
    {func_info,{atom,'Elixir.Test'},{atom,start},0}.
  {label,10}.
    {move,{atom,hello},{x,1}}.
    {move,{literal,[1]},{x,2}}.
    {move,{atom,'Elixir.Test'},{x,0}}.
    {line,[{location,"spawn.ex",4}]}.
    {call_ext_only,3,{extfunc,erlang,spawn,3}}.


{function, module_info, 0, 12}.
  {label,11}.
    {line,[]}.
    {func_info,{atom,'Elixir.Test'},{atom,module_info},0}.
  {label,12}.
    {move,{atom,'Elixir.Test'},{x,0}}.
    {line,[]}.
    {call_ext_only,1,{extfunc,erlang,get_module_info,1}}.


{function, module_info, 1, 14}.
  {label,13}.
    {line,[]}.
    {func_info,{atom,'Elixir.Test'},{atom,module_info},1}.
  {label,14}.
    {move,{x,0},{x,1}}.
    {move,{atom,'Elixir.Test'},{x,0}}.
    {line,[]}.
    {call_ext_only,2,{extfunc,erlang,get_module_info,2}}.
