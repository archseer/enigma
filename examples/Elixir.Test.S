{module, 'Elixir.Test'}.  %% version = 0

{exports, [{'__info__',1},{hello,1},{module_info,0},{module_info,1},{start,0}]}.

{attributes, []}.

{labels, 20}.


{function, '__info__', 1, 2}.
  {label,1}.
    {line,[]}.
    {func_info,{atom,'Elixir.Test'},{atom,'__info__'},1}.
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
    {line,[{location,"spawn.ex",12}]}.
    {func_info,{atom,'Elixir.Test'},{atom,hello},1}.
  {label,8}.
    {move,{literal,{hello,[1,2,3]}},{x,1}}.
    {line,[{location,"spawn.ex",14}]}.
    {call_ext_only,2,{extfunc,erlang,send,2}}.


{function, start, 0, 10}.
  {label,9}.
    {line,[{location,"spawn.ex",3}]}.
    {func_info,{atom,'Elixir.Test'},{atom,start},0}.
  {label,10}.
    {allocate_heap,0,2,0}.
    {bif,self,{f,0},[],{x,0}}.
    {put_list,{x,0},nil,{x,2}}.
    {move,{atom,hello},{x,1}}.
    {move,{atom,'Elixir.Test'},{x,0}}.
    {line,[{location,"spawn.ex",5}]}.
    {call_ext,3,{extfunc,erlang,spawn,3}}.
    {line,[{location,"spawn.ex",6}]}.
  {label,11}.
    {loop_rec,{f,15},{x,0}}.
    {test,is_tuple,{f,14},[{x,0}]}.
    {test,test_arity,{f,14},[{x,0},2]}.
    {get_tuple_element,{x,0},0,{x,1}}.
    {get_tuple_element,{x,0},1,{x,2}}.
    {test,is_atom,{f,14},[{x,1}]}.
    {select_val,{x,1},{f,14},{list,[{atom,hello},{f,12},{atom,world},{f,13}]}}.
  {label,12}.
    remove_message.
    {move,{x,2},{x,0}}.
    {deallocate,0}.
    return.
  {label,13}.
    remove_message.
    {move,{atom,no_match},{x,0}}.
    {deallocate,0}.
    return.
  {label,14}.
    {loop_rec_end,{f,11}}.
  {label,15}.
    {wait,{f,11}}.


{function, module_info, 0, 17}.
  {label,16}.
    {line,[]}.
    {func_info,{atom,'Elixir.Test'},{atom,module_info},0}.
  {label,17}.
    {move,{atom,'Elixir.Test'},{x,0}}.
    {line,[]}.
    {call_ext_only,1,{extfunc,erlang,get_module_info,1}}.


{function, module_info, 1, 19}.
  {label,18}.
    {line,[]}.
    {func_info,{atom,'Elixir.Test'},{atom,module_info},1}.
  {label,19}.
    {move,{x,0},{x,1}}.
    {move,{atom,'Elixir.Test'},{x,0}}.
    {line,[]}.
    {call_ext_only,2,{extfunc,erlang,get_module_info,2}}.
