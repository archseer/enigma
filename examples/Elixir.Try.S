{module, 'Elixir.Try'}.  %% version = 0

{exports, [{'__info__',1},
           {except,0},
           {module_info,0},
           {module_info,1},
           {start,0},
           {start2,0}]}.

{attributes, []}.

{labels, 29}.


{function, '__info__', 1, 2}.
  {label,1}.
    {line,[]}.
    {func_info,{atom,'Elixir.Try'},{atom,'__info__'},1}.
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
    {move,{atom,'Elixir.Try'},{x,0}}.
    {line,[]}.
    {call_ext_only,2,{extfunc,erlang,get_module_info,2}}.
  {label,4}.
    {move,{literal,[{except,0},{start,0},{start2,0}]},{x,0}}.
    return.
  {label,5}.
    {move,{atom,'Elixir.Try'},{x,0}}.
    return.
  {label,6}.
    {move,nil,{x,0}}.
    return.


{function, except, 0, 8}.
  {label,7}.
    {line,[{location,"trycatch.ex",18}]}.
    {func_info,{atom,'Elixir.Try'},{atom,except},0}.
  {label,8}.
    {allocate_zero,2,0}.
    {'try',{y,1},{f,15}}.
    {move,{integer,1},{x,0}}.
    {'try',{y,0},{f,9}}.
    {line,[{location,"trycatch.ex",20}]}.
    {call_ext,1,{extfunc,'Elixir.Kernel.Utils',raise,1}}.
    {line,[{location,"trycatch.ex",20}]}.
    {call_ext,1,{extfunc,erlang,error,1}}.
  {label,9}.
    {try_case,{y,0}}.
    {test,is_eq_exact,{f,13},[{x,0},{atom,error}]}.
    {move,{x,1},{y,0}}.
    {move,{x,2},{x,0}}.
    build_stacktrace.
    {test,is_tuple,{f,10},[{y,0}]}.
    {test,test_arity,{f,10},[{y,0},2]}.
    {bif,element,{f,10},[{integer,1},{y,0}],{x,1}}.
    {test,is_ne_exact,{f,12},[{x,1},{atom,badkey}]}.
  {label,10}.
    {select_val,{y,0},
                {f,11},
                {list,[{atom,undef},{f,12},{atom,function_clause},{f,12}]}}.
  {label,11}.
    {move,nil,{x,0}}.
  {label,12}.
    {move,{y,0},{x,1}}.
    {move,{x,0},{x,2}}.
    {kill,{y,0}}.
    {move,{atom,error},{x,0}}.
    {line,[{location,"trycatch.ex",22}]}.
    {call_ext,3,{extfunc,'Elixir.Exception',normalize,3}}.
    {move,{integer,2},{x,0}}.
    {jump,{f,14}}.
  {label,13}.
    {line,[]}.
    {bif,raise,{f,0},[{x,2},{x,1}],{x,0}}.
  {label,14}.
    {try_end,{y,1}}.
    {move,{x,0},{y,1}}.
    {line,[{location,"trycatch.ex",19}]}.
    {call,0,{f,28}}.
    {move,{y,1},{x,0}}.
    {deallocate,2}.
    return.
  {label,15}.
    {try_case,{y,1}}.
    {move,{x,2},{y,0}}.
    {move,{x,1},{y,1}}.
    {line,[{location,"trycatch.ex",19}]}.
    {call,0,{f,28}}.
    {line,[]}.
    {bif,raise,{f,0},[{y,0},{y,1}],{x,0}}.


{function, start, 0, 17}.
  {label,16}.
    {line,[{location,"trycatch.ex",3}]}.
    {func_info,{atom,'Elixir.Try'},{atom,start},0}.
  {label,17}.
    {move,{atom,ok},{x,0}}.
    {line,[{location,"trycatch.ex",4}]}.
    {call_ext,1,{extfunc,erlang,throw,1}}.


{function, start2, 0, 19}.
  {label,18}.
    {line,[{location,"trycatch.ex",7}]}.
    {func_info,{atom,'Elixir.Try'},{atom,start2},0}.
  {label,19}.
    {allocate,1,0}.
    {move,{integer,2},{x,0}}.
    {'try',{y,0},{f,20}}.
    {line,[{location,"trycatch.ex",10}]}.
    {call_ext,1,{extfunc,erlang,throw,1}}.
  {label,20}.
    {try_case,{y,0}}.
    {test,is_eq_exact,{f,22},[{x,0},{atom,throw}]}.
    {test,is_eq_exact,{f,21},[{x,1},{integer,1}]}.
    {move,{integer,1},{x,0}}.
    {deallocate,1}.
    return.
  {label,21}.
    {line,[{location,"trycatch.ex",13}]}.
    {gc_bif,'+',{f,0},2,[{x,1},{integer,1}],{x,0}}.
    {deallocate,1}.
    return.
  {label,22}.
    {line,[]}.
    {bif,raise,{f,0},[{x,2},{x,1}],{x,0}}.


{function, module_info, 0, 24}.
  {label,23}.
    {line,[]}.
    {func_info,{atom,'Elixir.Try'},{atom,module_info},0}.
  {label,24}.
    {move,{atom,'Elixir.Try'},{x,0}}.
    {line,[]}.
    {call_ext_only,1,{extfunc,erlang,get_module_info,1}}.


{function, module_info, 1, 26}.
  {label,25}.
    {line,[]}.
    {func_info,{atom,'Elixir.Try'},{atom,module_info},1}.
  {label,26}.
    {move,{x,0},{x,1}}.
    {move,{atom,'Elixir.Try'},{x,0}}.
    {line,[]}.
    {call_ext_only,2,{extfunc,erlang,get_module_info,2}}.


{function, '-except/0-after$^0/0-0-', 0, 28}.
  {label,27}.
    {line,[{location,"trycatch.ex",19}]}.
    {func_info,{atom,'Elixir.Try'},{atom,'-except/0-after$^0/0-0-'},0}.
  {label,28}.
    {move,{integer,3},{x,0}}.
    return.
