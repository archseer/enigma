{module, hello}.  %% version = 0

{exports, [{hello,0},{module_info,0},{module_info,1},{one,0},{world,0}]}.

{attributes, []}.

{labels, 11}.


{function, one, 0, 2}.
  {label,1}.
    {line,[{location,"hello.erl",5}]}.
    {func_info,{atom,hello},{atom,one},0}.
  {label,2}.
    {move,{integer,1},{x,0}}.
    return.


{function, world, 0, 4}.
  {label,3}.
    {line,[{location,"hello.erl",8}]}.
    {func_info,{atom,hello},{atom,world},0}.
  {label,4}.
    {allocate_zero,1,0}.
    {line,[{location,"hello.erl",9}]}.
    {call,0,{f,2}}.
    {move,{x,0},{y,0}}.
    {line,[{location,"hello.erl",9}]}.
    {call,0,{f,2}}.
    {line,[{location,"hello.erl",9}]}.
    {gc_bif,'+',{f,0},1,[{y,0},{x,0}],{x,0}}.
    {deallocate,1}.
    return.


{function, hello, 0, 6}.
  {label,5}.
    {line,[{location,"hello.erl",11}]}.
    {func_info,{atom,hello},{atom,hello},0}.
  {label,6}.
    {move,{literal,[a,b,c]},{x,0}}.
    return.


{function, module_info, 0, 8}.
  {label,7}.
    {line,[]}.
    {func_info,{atom,hello},{atom,module_info},0}.
  {label,8}.
    {move,{atom,hello},{x,0}}.
    {line,[]}.
    {call_ext_only,1,{extfunc,erlang,get_module_info,1}}.


{function, module_info, 1, 10}.
  {label,9}.
    {line,[]}.
    {func_info,{atom,hello},{atom,module_info},1}.
  {label,10}.
    {move,{x,0},{x,1}}.
    {move,{atom,hello},{x,0}}.
    {line,[]}.
    {call_ext_only,2,{extfunc,erlang,get_module_info,2}}.
