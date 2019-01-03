{module, 'Elixir.Try'}.  %% version = 0

{exports, [{'__info__',1},
           {a,0},
           {b,0},
           {c,0},
           {except,0},
           {module_info,0},
           {module_info,1},
           {start,0},
           {start2,0}]}.

{attributes, []}.

{labels, 39}.


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
    {move,{literal,[{a,0},{b,0},{c,0},{except,0},{start,0},{start2,0}]},{x,0}}.
    return.
  {label,5}.
    {move,{atom,'Elixir.Try'},{x,0}}.
    return.
  {label,6}.
    {move,nil,{x,0}}.
    return.


{function, a, 0, 8}.
  {label,7}.
    {line,[{location,"trycatch.ex",12}]}.
    {func_info,{atom,'Elixir.Try'},{atom,a},0}.
  {label,8}.
    {allocate,0,0}.
    {line,[{location,"trycatch.ex",13}]}.
    {call,0,{f,10}}.
    {move,{integer,1},{x,0}}.
    {deallocate,0}.
    return.


{function, b, 0, 10}.
  {label,9}.
    {line,[{location,"trycatch.ex",17}]}.
    {func_info,{atom,'Elixir.Try'},{atom,b},0}.
  {label,10}.
    {allocate,0,0}.
    {line,[{location,"trycatch.ex",18}]}.
    {call,0,{f,12}}.
    {move,{integer,1},{x,0}}.
    {deallocate,0}.
    return.


{function, c, 0, 12}.
  {label,11}.
    {line,[{location,"trycatch.ex",22}]}.
    {func_info,{atom,'Elixir.Try'},{atom,c},0}.
  {label,12}.
    {allocate,1,0}.
    {move,{integer,2},{x,0}}.
    {'try',{y,0},{f,13}}.
    {line,[{location,"trycatch.ex",25}]}.
    {call_ext,1,{extfunc,erlang,throw,1}}.
  {label,13}.
    {try_case,{y,0}}.
    {test,is_eq_exact,{f,16},[{x,0},{atom,throw}]}.
    {test,is_integer,{f,16},[{x,1}]}.
    {select_val,{x,1},{f,16},{list,[{integer,3},{f,14},{integer,1},{f,15}]}}.
  {label,14}.
    {move,{integer,2},{x,0}}.
    {deallocate,1}.
    return.
  {label,15}.
    {move,{integer,1},{x,0}}.
    {deallocate,1}.
    return.
  {label,16}.
    {line,[]}.
    {bif,raise,{f,0},[{x,2},{x,1}],{x,0}}.


{function, except, 0, 18}.
  {label,17}.
    {line,[{location,"trycatch.ex",43}]}.
    {func_info,{atom,'Elixir.Try'},{atom,except},0}.
  {label,18}.
    {allocate_zero,2,0}.
    {'try',{y,1},{f,25}}.
    {move,{integer,1},{x,0}}.
    {'try',{y,0},{f,19}}.
    {line,[{location,"trycatch.ex",45}]}.
    {call_ext,1,{extfunc,'Elixir.Kernel.Utils',raise,1}}.
    {line,[{location,"trycatch.ex",45}]}.
    {call_ext,1,{extfunc,erlang,error,1}}.
  {label,19}.
    {try_case,{y,0}}.
    {test,is_eq_exact,{f,23},[{x,0},{atom,error}]}.
    {move,{x,1},{y,0}}.
    {move,{x,2},{x,0}}.
    build_stacktrace.
    {test,is_tuple,{f,20},[{y,0}]}.
    {test,test_arity,{f,20},[{y,0},2]}.
    {bif,element,{f,20},[{integer,1},{y,0}],{x,1}}.
    {test,is_ne_exact,{f,22},[{x,1},{atom,badkey}]}.
  {label,20}.
    {select_val,{y,0},
                {f,21},
                {list,[{atom,undef},{f,22},{atom,function_clause},{f,22}]}}.
  {label,21}.
    {move,nil,{x,0}}.
  {label,22}.
    {move,{y,0},{x,1}}.
    {move,{x,0},{x,2}}.
    {kill,{y,0}}.
    {move,{atom,error},{x,0}}.
    {line,[{location,"trycatch.ex",47}]}.
    {call_ext,3,{extfunc,'Elixir.Exception',normalize,3}}.
    {move,{integer,2},{x,0}}.
    {jump,{f,24}}.
  {label,23}.
    {line,[]}.
    {bif,raise,{f,0},[{x,2},{x,1}],{x,0}}.
  {label,24}.
    {try_end,{y,1}}.
    {move,{x,0},{y,1}}.
    {line,[{location,"trycatch.ex",44}]}.
    {call,0,{f,38}}.
    {move,{y,1},{x,0}}.
    {deallocate,2}.
    return.
  {label,25}.
    {try_case,{y,1}}.
    {move,{x,2},{y,0}}.
    {move,{x,1},{y,1}}.
    {line,[{location,"trycatch.ex",44}]}.
    {call,0,{f,38}}.
    {line,[]}.
    {bif,raise,{f,0},[{y,0},{y,1}],{x,0}}.


{function, start, 0, 27}.
  {label,26}.
    {line,[{location,"trycatch.ex",7}]}.
    {func_info,{atom,'Elixir.Try'},{atom,start},0}.
  {label,27}.
    {allocate,0,0}.
    {line,[{location,"trycatch.ex",8}]}.
    {call,0,{f,8}}.
    {move,{integer,1},{x,0}}.
    {deallocate,0}.
    return.


{function, start2, 0, 29}.
  {label,28}.
    {line,[{location,"trycatch.ex",32}]}.
    {func_info,{atom,'Elixir.Try'},{atom,start2},0}.
  {label,29}.
    {allocate,1,0}.
    {move,{integer,2},{x,0}}.
    {'try',{y,0},{f,30}}.
    {line,[{location,"trycatch.ex",35}]}.
    {call_ext,1,{extfunc,erlang,throw,1}}.
  {label,30}.
    {try_case,{y,0}}.
    {test,is_eq_exact,{f,32},[{x,0},{atom,throw}]}.
    {test,is_eq_exact,{f,31},[{x,1},{integer,1}]}.
    {move,{integer,1},{x,0}}.
    {deallocate,1}.
    return.
  {label,31}.
    {line,[{location,"trycatch.ex",38}]}.
    {gc_bif,'+',{f,0},2,[{x,1},{integer,1}],{x,0}}.
    {deallocate,1}.
    return.
  {label,32}.
    {line,[]}.
    {bif,raise,{f,0},[{x,2},{x,1}],{x,0}}.


{function, module_info, 0, 34}.
  {label,33}.
    {line,[]}.
    {func_info,{atom,'Elixir.Try'},{atom,module_info},0}.
  {label,34}.
    {move,{atom,'Elixir.Try'},{x,0}}.
    {line,[]}.
    {call_ext_only,1,{extfunc,erlang,get_module_info,1}}.


{function, module_info, 1, 36}.
  {label,35}.
    {line,[]}.
    {func_info,{atom,'Elixir.Try'},{atom,module_info},1}.
  {label,36}.
    {move,{x,0},{x,1}}.
    {move,{atom,'Elixir.Try'},{x,0}}.
    {line,[]}.
    {call_ext_only,2,{extfunc,erlang,get_module_info,2}}.


{function, '-except/0-after$^0/0-0-', 0, 38}.
  {label,37}.
    {line,[{location,"trycatch.ex",44}]}.
    {func_info,{atom,'Elixir.Try'},{atom,'-except/0-after$^0/0-0-'},0}.
  {label,38}.
    {move,{integer,3},{x,0}}.
    return.
