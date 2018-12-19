{module, spw}.  %% version = 0

{exports, [{hello,1},{module_info,0},{module_info,1},{start,0}]}.

{attributes, []}.

{labels, 9}.


{function, start, 0, 2}.
  {label,1}.
    {line,[{location,"spw.erl",4}]}.
    {func_info,{atom,spw},{atom,start},0}.
  {label,2}.
    {move,{atom,hello},{x,1}}.
    {move,{literal,[1]},{x,2}}.
    {move,{atom,spw},{x,0}}.
    {line,[{location,"spw.erl",5}]}.
    {call_ext_only,3,{extfunc,erlang,spawn,3}}.


{function, hello, 1, 4}.
  {label,3}.
    {line,[{location,"spw.erl",7}]}.
    {func_info,{atom,spw},{atom,hello},1}.
  {label,4}.
    {test,is_eq_exact,{f,3},[{x,0},{atom,x}]}.
    return.


{function, module_info, 0, 6}.
  {label,5}.
    {line,[]}.
    {func_info,{atom,spw},{atom,module_info},0}.
  {label,6}.
    {move,{atom,spw},{x,0}}.
    {line,[]}.
    {call_ext_only,1,{extfunc,erlang,get_module_info,1}}.


{function, module_info, 1, 8}.
  {label,7}.
    {line,[]}.
    {func_info,{atom,spw},{atom,module_info},1}.
  {label,8}.
    {move,{x,0},{x,1}}.
    {move,{atom,spw},{x,0}}.
    {line,[]}.
    {call_ext_only,2,{extfunc,erlang,get_module_info,2}}.
