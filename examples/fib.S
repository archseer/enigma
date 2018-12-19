{module, fib}.  %% version = 0

{exports, [{fib,1},{module_info,0},{module_info,1}]}.

{attributes, []}.

{labels, 9}.


{function, fib, 1, 2}.
  {label,1}.
    {line,[{location,"fib.erl",4}]}.
    {func_info,{atom,fib},{atom,fib},1}.
  {label,2}.
    {test,is_eq,{f,3},[{x,0},{integer,0}]}.
    {move,{integer,0},{x,0}}.
    return.
  {label,3}.
    {test,is_eq,{f,4},[{x,0},{integer,1}]}.
    {move,{integer,1},{x,0}}.
    return.
  {label,4}.
    {allocate_zero,1,1}.
    {line,[{location,"fib.erl",9}]}.
    {gc_bif,'-',{f,0},1,[{x,0},{integer,1}],{x,1}}.
    {move,{x,0},{y,0}}.
    {move,{x,1},{x,0}}.
    {line,[{location,"fib.erl",9}]}.
    {call,1,{f,2}}.
    {line,[{location,"fib.erl",9}]}.
    {gc_bif,'-',{f,0},1,[{y,0},{integer,2}],{x,1}}.
    {move,{x,0},{y,0}}.
    {move,{x,1},{x,0}}.
    {line,[{location,"fib.erl",9}]}.
    {call,1,{f,2}}.
    {line,[{location,"fib.erl",9}]}.
    {gc_bif,'+',{f,0},1,[{y,0},{x,0}],{x,0}}.
    {deallocate,1}.
    return.


{function, module_info, 0, 6}.
  {label,5}.
    {line,[]}.
    {func_info,{atom,fib},{atom,module_info},0}.
  {label,6}.
    {move,{atom,fib},{x,0}}.
    {line,[]}.
    {call_ext_only,1,{extfunc,erlang,get_module_info,1}}.


{function, module_info, 1, 8}.
  {label,7}.
    {line,[]}.
    {func_info,{atom,fib},{atom,module_info},1}.
  {label,8}.
    {move,{x,0},{x,1}}.
    {move,{atom,fib},{x,0}}.
    {line,[]}.
    {call_ext_only,2,{extfunc,erlang,get_module_info,2}}.
