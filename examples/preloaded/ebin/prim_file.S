{module, prim_file}.  %% version = 0

{exports, [{advise,4},
           {allocate,3},
           {altname,1},
           {close,1},
           {copy,3},
           {datasync,1},
           {del_dir,1},
           {delete,1},
           {get_cwd,0},
           {get_cwd,1},
           {get_handle,1},
           {internal_name2native,1},
           {internal_native2name,1},
           {internal_normalize_utf8,1},
           {ipread_s32bu_p32bu,3},
           {is_translatable,1},
           {list_dir,1},
           {list_dir_all,1},
           {make_dir,1},
           {make_link,2},
           {make_symlink,2},
           {module_info,0},
           {module_info,1},
           {on_load,0},
           {open,2},
           {position,2},
           {pread,2},
           {pread,3},
           {pwrite,2},
           {pwrite,3},
           {read,2},
           {read_file,1},
           {read_file_info,1},
           {read_file_info,2},
           {read_line,1},
           {read_link,1},
           {read_link_all,1},
           {read_link_info,1},
           {read_link_info,2},
           {rename,2},
           {sendfile,8},
           {set_cwd,1},
           {sync,1},
           {truncate,1},
           {write,2},
           {write_file,2},
           {write_file_info,2},
           {write_file_info,3}]}.

{attributes, []}.

{labels, 444}.


{function, internal_name2native, 1, 2}.
  {label,1}.
    {line,[{location,"prim_file.erl",67}]}.
    {func_info,{atom,prim_file},{atom,internal_name2native},1}.
  {label,2}.
    {move,{atom,undefined},{x,0}}.
    {line,[{location,"prim_file.erl",68}]}.
    {call_ext_only,1,{extfunc,erlang,nif_error,1}}.


{function, internal_native2name, 1, 4}.
  {label,3}.
    {line,[{location,"prim_file.erl",73}]}.
    {func_info,{atom,prim_file},{atom,internal_native2name},1}.
  {label,4}.
    {move,{atom,undefined},{x,0}}.
    {line,[{location,"prim_file.erl",74}]}.
    {call_ext_only,1,{extfunc,erlang,nif_error,1}}.


{function, internal_normalize_utf8, 1, 6}.
  {label,5}.
    {line,[{location,"prim_file.erl",78}]}.
    {func_info,{atom,prim_file},{atom,internal_normalize_utf8},1}.
  {label,6}.
    {move,{atom,undefined},{x,0}}.
    {line,[{location,"prim_file.erl",79}]}.
    {call_ext_only,1,{extfunc,erlang,nif_error,1}}.


{function, is_translatable, 1, 8}.
  {label,7}.
    {line,[{location,"prim_file.erl",83}]}.
    {func_info,{atom,prim_file},{atom,is_translatable},1}.
  {label,8}.
    {move,{atom,undefined},{x,0}}.
    {line,[{location,"prim_file.erl",84}]}.
    {call_ext_only,1,{extfunc,erlang,nif_error,1}}.


{function, delayed_close_loop, 0, 10}.
  {label,9}.
    {line,[{location,"prim_file.erl",88}]}.
    {func_info,{atom,prim_file},{atom,delayed_close_loop},0}.
  {label,10}.
    {allocate,0,0}.
    {line,[{location,"prim_file.erl",89}]}.
  {label,11}.
    {loop_rec,{f,13},{x,0}}.
    {test,is_tagged_tuple,{f,12},[{x,0},2,{atom,close}]}.
    {get_tuple_element,{x,0},1,{x,2}}.
    {test,is_reference,{f,12},[{x,2}]}.
    remove_message.
    {move,{x,2},{x,0}}.
    {line,[{location,"prim_file.erl",90}]}.
    {call,1,{f,224}}.
    {jump,{f,14}}.
  {label,12}.
    remove_message.
    {jump,{f,14}}.
  {label,13}.
    {wait,{f,11}}.
  {label,14}.
    {call_last,0,{f,10},0}.


{function, copy, 3, 16}.
  {label,15}.
    {line,[{location,"prim_file.erl",98}]}.
    {func_info,{atom,prim_file},{atom,copy},3}.
  {label,16}.
    {test,is_tagged_tuple,{f,15},[{x,0},3,{atom,file_descriptor}]}.
    {get_tuple_element,{x,0},1,{x,4}}.
    {test,is_eq_exact,{f,15},[{x,4},{atom,prim_file}]}.
    {test,is_tagged_tuple,{f,15},[{x,1},3,{atom,file_descriptor}]}.
    {get_tuple_element,{x,1},1,{x,6}}.
    {test,is_eq_exact,{f,15},[{x,6},{atom,prim_file}]}.
    {test,is_integer,{f,17},[{x,2}]}.
    {test,is_lt,{f,18},[{x,2},{integer,0}]}.
  {label,17}.
    {test,is_atom,{f,15},[{x,2}]}.
  {label,18}.
    {line,[{location,"prim_file.erl",104}]}.
    {call_ext_only,3,{extfunc,file,copy_opened,3}}.


{function, on_load, 0, 20}.
  {label,19}.
    {line,[{location,"prim_file.erl",106}]}.
    {func_info,{atom,prim_file},{atom,on_load},0}.
  {label,20}.
    {allocate_zero,1,0}.
    {make_fun2,{f,443},0,0,0}.
    {line,[{location,"prim_file.erl",107}]}.
    {call_ext,1,{extfunc,erlang,spawn,1}}.
    {move,{x,0},{x,1}}.
    {move,{x,1},{y,0}}.
    {move,{atom,erts_prim_file},{x,0}}.
    {line,[{location,"prim_file.erl",111}]}.
    {call_ext,2,{extfunc,erlang,register,2}}.
    {test,is_eq_exact,{f,22},[{x,0},{atom,true}]}.
    {move,{y,0},{x,1}}.
    {trim,1,0}.
    {move,{literal,"prim_file"},{x,0}}.
    {line,[{location,"prim_file.erl",112}]}.
    {call_ext,2,{extfunc,erlang,load_nif,2}}.
    {test,is_eq_exact,{f,21},[{x,0},{atom,ok}]}.
    {deallocate,0}.
    return.
  {label,21}.
    {line,[{location,"prim_file.erl",112}]}.
    {badmatch,{x,0}}.
  {label,22}.
    {line,[{location,"prim_file.erl",111}]}.
    {badmatch,{x,0}}.


{function, open, 2, 24}.
  {label,23}.
    {line,[{location,"prim_file.erl",114}]}.
    {func_info,{atom,prim_file},{atom,open},2}.
  {label,24}.
    {allocate,2,2}.
    {move,{x,1},{y,0}}.
    {'try',{y,1},{f,27}}.
    {line,[{location,"prim_file.erl",118}]}.
    {call,1,{f,422}}.
    {move,{y,0},{x,1}}.
    {line,[{location,"prim_file.erl",118}]}.
    {call,2,{f,200}}.
    {try_end,{y,1}}.
    {test,is_tuple,{f,29},[{x,0}]}.
    {test,test_arity,{f,29},[{x,0},2]}.
    {get_tuple_element,{x,0},0,{x,1}}.
    {get_tuple_element,{x,0},1,{x,2}}.
    {test,is_atom,{f,29},[{x,1}]}.
    {select_val,{x,1},{f,29},{list,[{atom,ok},{f,25},{atom,error},{f,26}]}}.
  {label,25}.
    {move,{y,0},{x,1}}.
    {move,{x,2},{x,0}}.
    {line,[{location,"prim_file.erl",119}]}.
    {call,2,{f,31}}.
    {test_heap,3,1}.
    {put_tuple,2,{x,1}}.
    {put,{atom,ok}}.
    {put,{x,0}}.
    {move,{x,1},{x,0}}.
    {deallocate,2}.
    return.
  {label,26}.
    {deallocate,2}.
    return.
  {label,27}.
    {try_case,{y,1}}.
    {test,is_eq_exact,{f,28},[{x,0},{atom,error}]}.
    {test,is_eq_exact,{f,28},[{x,1},{atom,badarg}]}.
    {move,{literal,{error,badarg}},{x,0}}.
    {deallocate,2}.
    return.
  {label,28}.
    {line,[]}.
    {bif,raise,{f,0},[{x,2},{x,1}],{x,0}}.
  {label,29}.
    {line,[{location,"prim_file.erl",118}]}.
    {try_case_end,{x,0}}.


{function, make_fd, 2, 31}.
  {label,30}.
    {line,[{location,"prim_file.erl",125}]}.
    {func_info,{atom,prim_file},{atom,make_fd},2}.
  {label,31}.
    {allocate,0,2}.
    {line,[{location,"prim_file.erl",126}]}.
    {call,2,{f,192}}.
    {test_heap,4,1}.
    {put_tuple,3,{x,1}}.
    {put,{atom,file_descriptor}}.
    {put,{atom,prim_file}}.
    {put,{x,0}}.
    {move,{x,1},{x,0}}.
    {deallocate,0}.
    return.


{function, close, 1, 33}.
  {label,32}.
    {line,[{location,"prim_file.erl",128}]}.
    {func_info,{atom,prim_file},{atom,close},1}.
  {label,33}.
    {allocate,1,1}.
    {'try',{y,0},{f,34}}.
    {line,[{location,"prim_file.erl",130}]}.
    {call,1,{f,188}}.
    {test,is_map,{f,36},[{x,0}]}.
    {get_map_elements,{f,36},{x,0},{list,[{atom,handle},{x,1}]}}.
    {move,{x,1},{x,0}}.
    {line,[{location,"prim_file.erl",131}]}.
    {call,1,{f,202}}.
    {try_end,{y,0}}.
    {deallocate,1}.
    return.
  {label,34}.
    {try_case,{y,0}}.
    {test,is_eq_exact,{f,35},[{x,0},{atom,error}]}.
    {test,is_eq_exact,{f,35},[{x,1},{atom,badarg}]}.
    {move,{literal,{error,badarg}},{x,0}}.
    {deallocate,1}.
    return.
  {label,35}.
    {line,[]}.
    {bif,raise,{f,0},[{x,2},{x,1}],{x,0}}.
  {label,36}.
    {line,[{location,"prim_file.erl",130}]}.
    {badmatch,{x,0}}.


{function, read, 2, 38}.
  {label,37}.
    {line,[{location,"prim_file.erl",136}]}.
    {func_info,{atom,prim_file},{atom,read},2}.
  {label,38}.
    {allocate_zero,5,2}.
    {move,{x,1},{y,3}}.
    {'try',{y,4},{f,39}}.
    {line,[{location,"prim_file.erl",140}]}.
    {call,1,{f,188}}.
    {test,is_map,{f,41},[{x,0}]}.
    {get_map_elements,{f,41},
                      {x,0},
                      {list,[{atom,handle},
                             {x,3},
                             {atom,r_ahead_size},
                             {x,2},
                             {atom,r_buffer},
                             {x,1}]}}.
    {move,{x,1},{y,0}}.
    {move,{x,2},{y,1}}.
    {move,{x,3},{y,2}}.
    {move,{x,1},{x,0}}.
    {line,[{location,"prim_file.erl",141}]}.
    {call_ext,1,{extfunc,prim_buffer,size,1}}.
    {move,{x,0},{x,2}}.
    {move,{y,1},{x,3}}.
    {move,{y,0},{x,1}}.
    {move,{y,3},{x,4}}.
    {move,{y,2},{x,0}}.
    {kill,{y,0}}.
    {kill,{y,1}}.
    {kill,{y,2}}.
    {kill,{y,3}}.
    {line,[{location,"prim_file.erl",141}]}.
    {call,5,{f,43}}.
    {try_end,{y,4}}.
    {deallocate,5}.
    return.
  {label,39}.
    {try_case,{y,4}}.
    {test,is_eq_exact,{f,40},[{x,0},{atom,error}]}.
    {test,is_eq_exact,{f,40},[{x,1},{atom,badarg}]}.
    {move,{literal,{error,badarg}},{x,0}}.
    {deallocate,5}.
    return.
  {label,40}.
    {line,[]}.
    {bif,raise,{f,0},[{x,2},{x,1}],{x,0}}.
  {label,41}.
    {line,[{location,"prim_file.erl",140}]}.
    {badmatch,{x,0}}.


{function, read_1, 5, 43}.
  {label,42}.
    {line,[{location,"prim_file.erl",153}]}.
    {func_info,{atom,prim_file},{atom,read_1},5}.
  {label,43}.
    {test,is_ge,{f,44},[{x,2},{x,4}]}.
    {allocate,0,5}.
    {move,{x,1},{x,0}}.
    {move,{x,4},{x,1}}.
    {line,[{location,"prim_file.erl",154}]}.
    {call_ext,2,{extfunc,prim_buffer,read,2}}.
    {test_heap,3,1}.
    {put_tuple,2,{x,1}}.
    {put,{atom,ok}}.
    {put,{x,0}}.
    {move,{x,1},{x,0}}.
    {deallocate,0}.
    return.
  {label,44}.
    {test,is_lt,{f,48},[{integer,0},{x,2}]}.
    {allocate,5,5}.
    {move,{x,0},{y,4}}.
    {move,{x,1},{x,0}}.
    {move,{x,2},{x,1}}.
    {move,{x,4},{y,0}}.
    {move,{x,1},{y,1}}.
    {move,{x,3},{y,2}}.
    {move,{x,0},{y,3}}.
    {line,[{location,"prim_file.erl",156}]}.
    {call_ext,2,{extfunc,prim_buffer,read,2}}.
    {line,[{location,"prim_file.erl",157}]}.
    {gc_bif,'-',{f,0},1,[{y,0},{y,1}],{x,4}}.
    {move,{x,0},{x,5}}.
    {move,{y,2},{x,3}}.
    {move,{integer,0},{x,2}}.
    {move,{y,3},{x,1}}.
    {move,{y,4},{x,0}}.
    {move,{x,5},{y,4}}.
    {kill,{y,0}}.
    {kill,{y,1}}.
    {kill,{y,2}}.
    {kill,{y,3}}.
    {line,[{location,"prim_file.erl",157}]}.
    {call,5,{f,43}}.
    {test,is_tuple,{f,47},[{x,0}]}.
    {test,test_arity,{f,53},[{x,0},2]}.
    {get_tuple_element,{x,0},0,{x,1}}.
    {get_tuple_element,{x,0},1,{x,2}}.
    {test,is_atom,{f,53},[{x,1}]}.
    {select_val,{x,1},{f,53},{list,[{atom,ok},{f,45},{atom,error},{f,46}]}}.
  {label,45}.
    {line,[{location,"prim_file.erl",159}]}.
    {gc_bif,bit_size,{f,0},3,[{x,2}],{x,0}}.
    {bs_append,{f,0},{x,0},3,3,8,{y,4},{field_flags,[]},{x,0}}.
    {bs_put_binary,{f,0},{atom,all},8,{field_flags,[unsigned,big]},{x,2}}.
    {put_tuple,2,{x,1}}.
    {put,{atom,ok}}.
    {put,{x,0}}.
    {move,{x,1},{x,0}}.
    {deallocate,5}.
    return.
  {label,46}.
    {deallocate,5}.
    return.
  {label,47}.
    {test,is_eq_exact,{f,53},[{x,0},{atom,eof}]}.
    {test_heap,3,0}.
    {put_tuple,2,{x,0}}.
    {put,{atom,ok}}.
    {put,{y,4}}.
    {deallocate,5}.
    return.
  {label,48}.
    {test,is_eq_exact,{f,42},[{x,2},{integer,0}]}.
    {allocate_zero,2,5}.
    {line,[{location,"prim_file.erl",166}]}.
    {gc_bif,'+',{f,0},5,[{x,3},{x,4}],{x,2}}.
    {move,{x,1},{y,1}}.
    {move,{x,2},{x,1}}.
    {move,{x,4},{y,0}}.
    {line,[{location,"prim_file.erl",166}]}.
    {call,2,{f,204}}.
    {test,is_tuple,{f,51},[{x,0}]}.
    {test,test_arity,{f,55},[{x,0},2]}.
    {get_tuple_element,{x,0},0,{x,1}}.
    {get_tuple_element,{x,0},1,{x,2}}.
    {test,is_atom,{f,55},[{x,1}]}.
    {select_val,{x,1},{f,55},{list,[{atom,ok},{f,49},{atom,error},{f,52}]}}.
  {label,49}.
    {gc_bif,byte_size,{f,50},3,[{x,2}],{x,3}}.
    {test,is_lt,{f,50},[{y,0},{x,3}]}.
    {move,{y,0},{x,1}}.
    {kill,{y,0}}.
    {move,{x,2},{x,0}}.
    {line,[{location,"prim_file.erl",168}]}.
    {call_ext,2,{extfunc,erlang,split_binary,2}}.
    {test,is_tuple,{f,54},[{x,0}]}.
    {test,test_arity,{f,54},[{x,0},2]}.
    {test_heap,2,1}.
    {get_tuple_element,{x,0},0,{y,0}}.
    {get_tuple_element,{x,0},1,{x,2}}.
    {put_list,{x,2},nil,{x,1}}.
    {move,{y,1},{x,0}}.
    {kill,{y,1}}.
    {line,[{location,"prim_file.erl",169}]}.
    {call_ext,2,{extfunc,prim_buffer,write,2}}.
    {test_heap,3,0}.
    {put_tuple,2,{x,0}}.
    {put,{atom,ok}}.
    {put,{y,0}}.
    {deallocate,2}.
    return.
  {label,50}.
    {gc_bif,byte_size,{f,55},3,[{x,2}],{x,3}}.
    {test,is_ge,{f,55},[{y,0},{x,3}]}.
    {jump,{f,52}}.
  {label,51}.
    {test,is_eq_exact,{f,55},[{x,0},{atom,eof}]}.
  {label,52}.
    {deallocate,2}.
    return.
  {label,53}.
    {line,[{location,"prim_file.erl",157}]}.
    {case_end,{x,0}}.
  {label,54}.
    {line,[{location,"prim_file.erl",168}]}.
    {badmatch,{x,0}}.
  {label,55}.
    {line,[{location,"prim_file.erl",166}]}.
    {case_end,{x,0}}.


{function, read_line, 1, 57}.
  {label,56}.
    {line,[{location,"prim_file.erl",179}]}.
    {func_info,{atom,prim_file},{atom,read_line},1}.
  {label,57}.
    {allocate_zero,4,1}.
    {'try',{y,3},{f,58}}.
    {line,[{location,"prim_file.erl",183}]}.
    {call,1,{f,188}}.
    {test,is_map,{f,60},[{x,0}]}.
    {get_map_elements,{f,60},
                      {x,0},
                      {list,[{atom,handle},
                             {x,3},
                             {atom,r_ahead_size},
                             {x,2},
                             {atom,r_buffer},
                             {x,1}]}}.
    {move,{x,1},{x,0}}.
    {move,{integer,10},{x,1}}.
    {move,{x,2},{y,0}}.
    {move,{x,0},{y,1}}.
    {move,{x,3},{y,2}}.
    {line,[{location,"prim_file.erl",184}]}.
    {call_ext,2,{extfunc,prim_buffer,find_byte_index,2}}.
    {move,{x,0},{x,2}}.
    {move,{y,0},{x,1}}.
    {move,{x,2},{y,0}}.
    {move,{integer,256},{x,0}}.
    {line,[{location,"prim_file.erl",185}]}.
    {call_ext,2,{extfunc,erlang,max,2}}.
    {move,{y,0},{x,2}}.
    {move,{y,1},{x,1}}.
    {move,{x,0},{x,3}}.
    {move,{y,2},{x,0}}.
    {kill,{y,0}}.
    {kill,{y,1}}.
    {kill,{y,2}}.
    {line,[{location,"prim_file.erl",186}]}.
    {call,4,{f,62}}.
    {try_end,{y,3}}.
    {deallocate,4}.
    return.
  {label,58}.
    {try_case,{y,3}}.
    {test,is_eq_exact,{f,59},[{x,0},{atom,error}]}.
    {test,is_eq_exact,{f,59},[{x,1},{atom,badarg}]}.
    {move,{literal,{error,badarg}},{x,0}}.
    {deallocate,4}.
    return.
  {label,59}.
    {line,[]}.
    {bif,raise,{f,0},[{x,2},{x,1}],{x,0}}.
  {label,60}.
    {line,[{location,"prim_file.erl",183}]}.
    {badmatch,{x,0}}.


{function, read_line_1, 4, 62}.
  {label,61}.
    {line,[{location,"prim_file.erl",197}]}.
    {func_info,{atom,prim_file},{atom,read_line_1},4}.
  {label,62}.
    {test,is_tuple,{f,64},[{x,2}]}.
    {test,test_arity,{f,61},[{x,2},2]}.
    {get_tuple_element,{x,2},0,{x,4}}.
    {test,is_eq_exact,{f,61},[{x,4},{atom,ok}]}.
    {allocate_zero,1,5}.
    {get_tuple_element,{x,2},1,{x,5}}.
    {line,[{location,"prim_file.erl",213}]}.
    {gc_bif,'-',{f,0},6,[{x,5},{integer,1}],{x,0}}.
    {line,[{location,"prim_file.erl",214}]}.
    {gc_bif,'+',{f,0},6,[{x,5},{integer,1}],{x,2}}.
    {move,{x,0},{y,0}}.
    {move,{x,1},{x,0}}.
    {move,{x,2},{x,1}}.
    {line,[{location,"prim_file.erl",214}]}.
    {call_ext,2,{extfunc,prim_buffer,read,2}}.
    {test,bs_start_match2,{f,63},1,[{x,0},0],{x,1}}.
    {test,bs_get_binary2,
          {f,63},
          2,
          [{x,1},
           {y,0},
           8,
           {field_flags,[{anno,[215,{file,"prim_file.erl"}]},unsigned,big]}],
          {x,2}}.
    {test,bs_match_string,{f,63},[{x,1},16,{string,"\r\n"}]}.
    {test,bs_test_tail2,{f,63},[{x,1},0]}.
    {line,[{location,"prim_file.erl",215}]}.
    {bs_append,{f,0},{integer,8},3,0,8,{x,2},{field_flags,[]},{x,0}}.
    {bs_put_string,1,{string,"\n"}}.
    {put_tuple,2,{x,1}}.
    {put,{atom,ok}}.
    {put,{x,0}}.
    {move,{x,1},{x,0}}.
    {deallocate,1}.
    return.
  {label,63}.
    {test_heap,3,1}.
    {put_tuple,2,{x,1}}.
    {put,{atom,ok}}.
    {put,{x,0}}.
    {move,{x,1},{x,0}}.
    {deallocate,1}.
    return.
  {label,64}.
    {test,is_eq_exact,{f,61},[{x,2},{atom,not_found}]}.
    {allocate,3,4}.
    {move,{x,1},{y,1}}.
    {move,{x,3},{x,1}}.
    {move,{x,1},{y,0}}.
    {move,{x,0},{y,2}}.
    {line,[{location,"prim_file.erl",198}]}.
    {call,2,{f,204}}.
    {test,is_tuple,{f,67},[{x,0}]}.
    {test,test_arity,{f,70},[{x,0},2]}.
    {get_tuple_element,{x,0},0,{x,1}}.
    {get_tuple_element,{x,0},1,{x,2}}.
    {test,is_atom,{f,70},[{x,1}]}.
    {select_val,{x,1},{f,70},{list,[{atom,ok},{f,65},{atom,error},{f,66}]}}.
  {label,65}.
    {test_heap,2,3}.
    {put_list,{x,2},nil,{x,1}}.
    {move,{y,1},{x,0}}.
    {line,[{location,"prim_file.erl",200}]}.
    {call_ext,2,{extfunc,prim_buffer,write,2}}.
    {move,{integer,10},{x,1}}.
    {move,{y,1},{x,0}}.
    {line,[{location,"prim_file.erl",201}]}.
    {call_ext,2,{extfunc,prim_buffer,find_byte_index,2}}.
    {move,{x,0},{x,2}}.
    {move,{y,1},{x,1}}.
    {move,{y,0},{x,3}}.
    {move,{y,2},{x,0}}.
    {call_last,4,{f,62},3}.
  {label,66}.
    {deallocate,3}.
    return.
  {label,67}.
    {test,is_eq_exact,{f,70},[{x,0},{atom,eof}]}.
    {kill,{y,0}}.
    {kill,{y,2}}.
    {move,{y,1},{x,0}}.
    {line,[{location,"prim_file.erl",204}]}.
    {call_ext,1,{extfunc,prim_buffer,size,1}}.
    {test,is_lt,{f,68},[{integer,0},{x,0}]}.
    {move,{x,0},{x,1}}.
    {move,{y,1},{x,0}}.
    {trim,3,0}.
    {line,[{location,"prim_file.erl",205}]}.
    {call_ext,2,{extfunc,prim_buffer,read,2}}.
    {test_heap,3,1}.
    {put_tuple,2,{x,1}}.
    {put,{atom,ok}}.
    {put,{x,0}}.
    {move,{x,1},{x,0}}.
    {deallocate,0}.
    return.
  {label,68}.
    {test,is_eq_exact,{f,69},[{x,0},{integer,0}]}.
    {move,{atom,eof},{x,0}}.
    {deallocate,3}.
    return.
  {label,69}.
    {line,[{location,"prim_file.erl",204}]}.
    {case_end,{x,0}}.
  {label,70}.
    {line,[{location,"prim_file.erl",198}]}.
    {case_end,{x,0}}.


{function, write, 2, 72}.
  {label,71}.
    {line,[{location,"prim_file.erl",219}]}.
    {func_info,{atom,prim_file},{atom,write},2}.
  {label,72}.
    {allocate,3,2}.
    {move,{x,1},{y,0}}.
    {move,{x,0},{y,1}}.
    {'try',{y,2},{f,73}}.
    {line,[{location,"prim_file.erl",221}]}.
    {call,1,{f,188}}.
    {test,is_map,{f,75},[{x,0}]}.
    {get_map_elements,{f,75},{x,0},{list,[{atom,handle},{x,1}]}}.
    {move,{y,1},{x,0}}.
    {move,{x,1},{y,1}}.
    {line,[{location,"prim_file.erl",222}]}.
    {call,1,{f,182}}.
    {move,{y,0},{x,0}}.
    {kill,{y,0}}.
    {line,[{location,"prim_file.erl",223}]}.
    {call_ext,1,{extfunc,erlang,iolist_to_iovec,1}}.
    {move,{x,0},{x,1}}.
    {move,{y,1},{x,0}}.
    {kill,{y,1}}.
    {line,[{location,"prim_file.erl",223}]}.
    {call,2,{f,77}}.
    {try_end,{y,2}}.
    {deallocate,3}.
    return.
  {label,73}.
    {try_case,{y,2}}.
    {test,is_eq_exact,{f,74},[{x,0},{atom,error}]}.
    {test,is_eq_exact,{f,74},[{x,1},{atom,badarg}]}.
    {move,{literal,{error,badarg}},{x,0}}.
    {deallocate,3}.
    return.
  {label,74}.
    {line,[]}.
    {bif,raise,{f,0},[{x,2},{x,1}],{x,0}}.
  {label,75}.
    {line,[{location,"prim_file.erl",221}]}.
    {badmatch,{x,0}}.


{function, write_1, 2, 77}.
  {label,76}.
    {line,[{location,"prim_file.erl",227}]}.
    {func_info,{atom,prim_file},{atom,write_1},2}.
  {label,77}.
    {allocate,1,2}.
    {move,{x,0},{y,0}}.
    {line,[{location,"prim_file.erl",228}]}.
    {call,2,{f,206}}.
    {test,is_tuple,{f,79},[{x,0}]}.
    {test,test_arity,{f,81},[{x,0},2]}.
    {get_tuple_element,{x,0},0,{x,1}}.
    {get_tuple_element,{x,0},1,{x,2}}.
    {test,is_atom,{f,81},[{x,1}]}.
    {select_val,{x,1},
                {f,81},
                {list,[{atom,continue},{f,78},{atom,error},{f,80}]}}.
  {label,78}.
    {move,{x,2},{x,1}}.
    {move,{y,0},{x,0}}.
    {call_last,2,{f,77},1}.
  {label,79}.
    {test,is_eq_exact,{f,81},[{x,0},{atom,ok}]}.
  {label,80}.
    {deallocate,1}.
    return.
  {label,81}.
    {line,[{location,"prim_file.erl",228}]}.
    {case_end,{x,0}}.


{function, truncate, 1, 83}.
  {label,82}.
    {line,[{location,"prim_file.erl",237}]}.
    {func_info,{atom,prim_file},{atom,truncate},1}.
  {label,83}.
    {allocate,2,1}.
    {move,{x,0},{y,0}}.
    {'try',{y,1},{f,84}}.
    {line,[{location,"prim_file.erl",239}]}.
    {call,1,{f,188}}.
    {test,is_map,{f,86},[{x,0}]}.
    {get_map_elements,{f,86},{x,0},{list,[{atom,handle},{x,1}]}}.
    {move,{y,0},{x,0}}.
    {move,{x,1},{y,0}}.
    {line,[{location,"prim_file.erl",240}]}.
    {call,1,{f,182}}.
    {move,{y,0},{x,0}}.
    {kill,{y,0}}.
    {line,[{location,"prim_file.erl",241}]}.
    {call,1,{f,220}}.
    {try_end,{y,1}}.
    {deallocate,2}.
    return.
  {label,84}.
    {try_case,{y,1}}.
    {test,is_eq_exact,{f,85},[{x,0},{atom,error}]}.
    {test,is_eq_exact,{f,85},[{x,1},{atom,badarg}]}.
    {move,{literal,{error,badarg}},{x,0}}.
    {deallocate,2}.
    return.
  {label,85}.
    {line,[]}.
    {bif,raise,{f,0},[{x,2},{x,1}],{x,0}}.
  {label,86}.
    {line,[{location,"prim_file.erl",239}]}.
    {badmatch,{x,0}}.


{function, advise, 4, 88}.
  {label,87}.
    {line,[{location,"prim_file.erl",246}]}.
    {func_info,{atom,prim_file},{atom,advise},4}.
  {label,88}.
    {allocate,4,4}.
    {move,{x,3},{y,0}}.
    {move,{x,2},{y,1}}.
    {move,{x,1},{y,2}}.
    {'try',{y,3},{f,89}}.
    {line,[{location,"prim_file.erl",248}]}.
    {call,1,{f,188}}.
    {test,is_map,{f,91},[{x,0}]}.
    {get_map_elements,{f,91},{x,0},{list,[{atom,handle},{x,1}]}}.
    {move,{y,1},{x,2}}.
    {move,{y,0},{x,3}}.
    {move,{x,1},{x,0}}.
    {move,{y,2},{x,1}}.
    {kill,{y,0}}.
    {kill,{y,1}}.
    {kill,{y,2}}.
    {line,[{location,"prim_file.erl",249}]}.
    {call,4,{f,216}}.
    {try_end,{y,3}}.
    {deallocate,4}.
    return.
  {label,89}.
    {try_case,{y,3}}.
    {test,is_eq_exact,{f,90},[{x,0},{atom,error}]}.
    {test,is_eq_exact,{f,90},[{x,1},{atom,badarg}]}.
    {move,{literal,{error,badarg}},{x,0}}.
    {deallocate,4}.
    return.
  {label,90}.
    {line,[]}.
    {bif,raise,{f,0},[{x,2},{x,1}],{x,0}}.
  {label,91}.
    {line,[{location,"prim_file.erl",248}]}.
    {badmatch,{x,0}}.


{function, allocate, 3, 93}.
  {label,92}.
    {line,[{location,"prim_file.erl",254}]}.
    {func_info,{atom,prim_file},{atom,allocate},3}.
  {label,93}.
    {allocate,3,3}.
    {move,{x,2},{y,0}}.
    {move,{x,1},{y,1}}.
    {'try',{y,2},{f,94}}.
    {line,[{location,"prim_file.erl",256}]}.
    {call,1,{f,188}}.
    {test,is_map,{f,96},[{x,0}]}.
    {get_map_elements,{f,96},{x,0},{list,[{atom,handle},{x,1}]}}.
    {move,{y,0},{x,2}}.
    {move,{x,1},{x,0}}.
    {move,{y,1},{x,1}}.
    {kill,{y,0}}.
    {kill,{y,1}}.
    {line,[{location,"prim_file.erl",257}]}.
    {call,3,{f,218}}.
    {try_end,{y,2}}.
    {deallocate,3}.
    return.
  {label,94}.
    {try_case,{y,2}}.
    {test,is_eq_exact,{f,95},[{x,0},{atom,error}]}.
    {test,is_eq_exact,{f,95},[{x,1},{atom,badarg}]}.
    {move,{literal,{error,badarg}},{x,0}}.
    {deallocate,3}.
    return.
  {label,95}.
    {line,[]}.
    {bif,raise,{f,0},[{x,2},{x,1}],{x,0}}.
  {label,96}.
    {line,[{location,"prim_file.erl",256}]}.
    {badmatch,{x,0}}.


{function, sync, 1, 98}.
  {label,97}.
    {line,[{location,"prim_file.erl",262}]}.
    {func_info,{atom,prim_file},{atom,sync},1}.
  {label,98}.
    {allocate,1,1}.
    {'try',{y,0},{f,99}}.
    {line,[{location,"prim_file.erl",264}]}.
    {call,1,{f,188}}.
    {test,is_map,{f,101},[{x,0}]}.
    {get_map_elements,{f,101},{x,0},{list,[{atom,handle},{x,1}]}}.
    {move,{x,1},{x,0}}.
    {move,{integer,0},{x,1}}.
    {line,[{location,"prim_file.erl",265}]}.
    {call,2,{f,214}}.
    {try_end,{y,0}}.
    {deallocate,1}.
    return.
  {label,99}.
    {try_case,{y,0}}.
    {test,is_eq_exact,{f,100},[{x,0},{atom,error}]}.
    {test,is_eq_exact,{f,100},[{x,1},{atom,badarg}]}.
    {move,{literal,{error,badarg}},{x,0}}.
    {deallocate,1}.
    return.
  {label,100}.
    {line,[]}.
    {bif,raise,{f,0},[{x,2},{x,1}],{x,0}}.
  {label,101}.
    {line,[{location,"prim_file.erl",264}]}.
    {badmatch,{x,0}}.


{function, datasync, 1, 103}.
  {label,102}.
    {line,[{location,"prim_file.erl",270}]}.
    {func_info,{atom,prim_file},{atom,datasync},1}.
  {label,103}.
    {allocate,1,1}.
    {'try',{y,0},{f,104}}.
    {line,[{location,"prim_file.erl",272}]}.
    {call,1,{f,188}}.
    {test,is_map,{f,106},[{x,0}]}.
    {get_map_elements,{f,106},{x,0},{list,[{atom,handle},{x,1}]}}.
    {move,{x,1},{x,0}}.
    {move,{integer,1},{x,1}}.
    {line,[{location,"prim_file.erl",273}]}.
    {call,2,{f,214}}.
    {try_end,{y,0}}.
    {deallocate,1}.
    return.
  {label,104}.
    {try_case,{y,0}}.
    {test,is_eq_exact,{f,105},[{x,0},{atom,error}]}.
    {test,is_eq_exact,{f,105},[{x,1},{atom,badarg}]}.
    {move,{literal,{error,badarg}},{x,0}}.
    {deallocate,1}.
    return.
  {label,105}.
    {line,[]}.
    {bif,raise,{f,0},[{x,2},{x,1}],{x,0}}.
  {label,106}.
    {line,[{location,"prim_file.erl",272}]}.
    {badmatch,{x,0}}.


{function, position, 2, 108}.
  {label,107}.
    {line,[{location,"prim_file.erl",278}]}.
    {func_info,{atom,prim_file},{atom,position},2}.
  {label,108}.
    {test,is_tuple,{f,114},[{x,1}]}.
    {test,test_arity,{f,118},[{x,1},2]}.
    {get_tuple_element,{x,1},0,{x,2}}.
    {get_tuple_element,{x,1},1,{x,3}}.
    {test,is_eq_exact,{f,111},[{x,2},{atom,cur}]}.
    {allocate,3,4}.
    {move,{x,0},{y,0}}.
    {move,{x,3},{y,1}}.
    {'try',{y,2},{f,109}}.
    {line,[{location,"prim_file.erl",281}]}.
    {call,1,{f,188}}.
    {test,is_map,{f,119},[{x,0}]}.
    {get_map_elements,{f,119},{x,0},{list,[{atom,r_buffer},{x,1}]}}.
    {move,{x,1},{x,0}}.
    {line,[{location,"prim_file.erl",282}]}.
    {call_ext,1,{extfunc,prim_buffer,size,1}}.
    {line,[{location,"prim_file.erl",282}]}.
    {gc_bif,'-',{f,0},1,[{y,1},{x,0}],{x,2}}.
    {move,{atom,cur},{x,1}}.
    {move,{y,0},{x,0}}.
    {kill,{y,0}}.
    {kill,{y,1}}.
    {line,[{location,"prim_file.erl",282}]}.
    {call,3,{f,121}}.
    {try_end,{y,2}}.
    {deallocate,3}.
    return.
  {label,109}.
    {try_case,{y,2}}.
    {test,is_eq_exact,{f,110},[{x,0},{atom,error}]}.
    {test,is_eq_exact,{f,110},[{x,1},{atom,badarg}]}.
    {move,{literal,{error,badarg}},{x,0}}.
    {deallocate,3}.
    return.
  {label,110}.
    {line,[]}.
    {bif,raise,{f,0},[{x,2},{x,1}],{x,0}}.
  {label,111}.
    {allocate,1,4}.
    {move,{x,2},{x,1}}.
    {move,{x,3},{x,2}}.
    {'try',{y,0},{f,112}}.
    {line,[{location,"prim_file.erl",288}]}.
    {call,3,{f,121}}.
    {try_end,{y,0}}.
    {deallocate,1}.
    return.
  {label,112}.
    {try_case,{y,0}}.
    {test,is_eq_exact,{f,113},[{x,0},{atom,error}]}.
    {test,is_eq_exact,{f,113},[{x,1},{atom,badarg}]}.
    {move,{literal,{error,badarg}},{x,0}}.
    {deallocate,1}.
    return.
  {label,113}.
    {line,[]}.
    {bif,raise,{f,0},[{x,2},{x,1}],{x,0}}.
  {label,114}.
    {test,is_atom,{f,118},[{x,1}]}.
    {select_val,{x,1},
                {f,118},
                {list,[{atom,eof},
                       {f,115},
                       {atom,cur},
                       {f,116},
                       {atom,bof},
                       {f,117}]}}.
  {label,115}.
    {move,{literal,{eof,0}},{x,1}}.
    {call_only,2,{f,108}}.
  {label,116}.
    {move,{literal,{cur,0}},{x,1}}.
    {call_only,2,{f,108}}.
  {label,117}.
    {move,{literal,{bof,0}},{x,1}}.
    {call_only,2,{f,108}}.
  {label,118}.
    {test_heap,3,2}.
    {put_tuple,2,{x,2}}.
    {put,{atom,bof}}.
    {put,{x,1}}.
    {move,{x,2},{x,1}}.
    {call_only,2,{f,108}}.
  {label,119}.
    {line,[{location,"prim_file.erl",281}]}.
    {badmatch,{x,0}}.


{function, position_1, 3, 121}.
  {label,120}.
    {line,[{location,"prim_file.erl",297}]}.
    {func_info,{atom,prim_file},{atom,position_1},3}.
  {label,121}.
    {allocate,3,3}.
    {init,{y,0}}.
    {move,{x,2},{y,1}}.
    {move,{x,1},{y,2}}.
    {line,[{location,"prim_file.erl",298}]}.
    {call,1,{f,188}}.
    {test,is_map,{f,122},[{x,0}]}.
    {get_map_elements,{f,122},
                      {x,0},
                      {list,[{atom,handle},{x,2},{atom,r_buffer},{x,1}]}}.
    {move,{x,2},{y,0}}.
    {move,{x,1},{x,0}}.
    {line,[{location,"prim_file.erl",299}]}.
    {call_ext,1,{extfunc,prim_buffer,wipe,1}}.
    {move,{y,2},{x,1}}.
    {move,{y,1},{x,2}}.
    {move,{y,0},{x,0}}.
    {call_last,3,{f,212},3}.
  {label,122}.
    {line,[{location,"prim_file.erl",298}]}.
    {badmatch,{x,0}}.


{function, pread, 3, 124}.
  {label,123}.
    {line,[{location,"prim_file.erl",302}]}.
    {func_info,{atom,prim_file},{atom,pread},3}.
  {label,124}.
    {allocate,3,3}.
    {move,{x,2},{y,0}}.
    {move,{x,1},{y,1}}.
    {'try',{y,2},{f,125}}.
    {line,[{location,"prim_file.erl",304}]}.
    {call,1,{f,188}}.
    {test,is_map,{f,127},[{x,0}]}.
    {get_map_elements,{f,127},{x,0},{list,[{atom,handle},{x,1}]}}.
    {move,{y,0},{x,2}}.
    {move,{x,1},{x,0}}.
    {move,{y,1},{x,1}}.
    {kill,{y,0}}.
    {kill,{y,1}}.
    {line,[{location,"prim_file.erl",305}]}.
    {call,3,{f,208}}.
    {try_end,{y,2}}.
    {deallocate,3}.
    return.
  {label,125}.
    {try_case,{y,2}}.
    {test,is_eq_exact,{f,126},[{x,0},{atom,error}]}.
    {test,is_eq_exact,{f,126},[{x,1},{atom,badarg}]}.
    {move,{literal,{error,badarg}},{x,0}}.
    {deallocate,3}.
    return.
  {label,126}.
    {line,[]}.
    {bif,raise,{f,0},[{x,2},{x,1}],{x,0}}.
  {label,127}.
    {line,[{location,"prim_file.erl",304}]}.
    {badmatch,{x,0}}.


{function, pread, 2, 129}.
  {label,128}.
    {line,[{location,"prim_file.erl",310}]}.
    {func_info,{atom,prim_file},{atom,pread},2}.
  {label,129}.
    {allocate,2,2}.
    {move,{x,1},{y,0}}.
    {'try',{y,1},{f,130}}.
    {line,[{location,"prim_file.erl",312}]}.
    {call,1,{f,188}}.
    {test,is_map,{f,132},[{x,0}]}.
    {get_map_elements,{f,132},{x,0},{list,[{atom,handle},{x,1}]}}.
    {move,nil,{x,2}}.
    {move,{x,1},{x,0}}.
    {move,{y,0},{x,1}}.
    {kill,{y,0}}.
    {line,[{location,"prim_file.erl",313}]}.
    {call,3,{f,134}}.
    {try_end,{y,1}}.
    {deallocate,2}.
    return.
  {label,130}.
    {try_case,{y,1}}.
    {test,is_eq_exact,{f,131},[{x,0},{atom,error}]}.
    {test,is_eq_exact,{f,131},[{x,1},{atom,badarg}]}.
    {move,{literal,{error,badarg}},{x,0}}.
    {deallocate,2}.
    return.
  {label,131}.
    {line,[]}.
    {bif,raise,{f,0},[{x,2},{x,1}],{x,0}}.
  {label,132}.
    {line,[{location,"prim_file.erl",312}]}.
    {badmatch,{x,0}}.


{function, pread_list, 3, 134}.
  {label,133}.
    {line,[{location,"prim_file.erl",324}]}.
    {func_info,{atom,prim_file},{atom,pread_list},3}.
  {label,134}.
    {test,is_nonempty_list,{f,138},[{x,1}]}.
    {get_list,{x,1},{x,3},{x,4}}.
    {test,is_tuple,{f,133},[{x,3}]}.
    {test,test_arity,{f,133},[{x,3},2]}.
    {allocate,3,5}.
    {get_tuple_element,{x,3},0,{x,1}}.
    {move,{x,2},{y,0}}.
    {get_tuple_element,{x,3},1,{x,2}}.
    {move,{x,0},{y,1}}.
    {move,{x,4},{y,2}}.
    {line,[{location,"prim_file.erl",327}]}.
    {call,3,{f,208}}.
    {test,is_tuple,{f,137},[{x,0}]}.
    {test,test_arity,{f,139},[{x,0},2]}.
    {get_tuple_element,{x,0},0,{x,1}}.
    {get_tuple_element,{x,0},1,{x,2}}.
    {test,is_atom,{f,139},[{x,1}]}.
    {select_val,{x,1},{f,139},{list,[{atom,ok},{f,135},{atom,error},{f,136}]}}.
  {label,135}.
    {test_heap,2,3}.
    {put_list,{x,2},{y,0},{x,2}}.
    {move,{y,2},{x,1}}.
    {move,{y,1},{x,0}}.
    {call_last,3,{f,134},3}.
  {label,136}.
    {deallocate,3}.
    return.
  {label,137}.
    {test,is_eq_exact,{f,139},[{x,0},{atom,eof}]}.
    {test_heap,2,0}.
    {put_list,{atom,eof},{y,0},{x,2}}.
    {move,{y,2},{x,1}}.
    {move,{y,1},{x,0}}.
    {call_last,3,{f,134},3}.
  {label,138}.
    {test,is_nil,{f,133},[{x,1}]}.
    {allocate,0,3}.
    {move,{x,2},{x,0}}.
    {line,[{location,"prim_file.erl",325}]}.
    {call,1,{f,415}}.
    {test_heap,3,1}.
    {put_tuple,2,{x,1}}.
    {put,{atom,ok}}.
    {put,{x,0}}.
    {move,{x,1},{x,0}}.
    {deallocate,0}.
    return.
  {label,139}.
    {line,[{location,"prim_file.erl",327}]}.
    {case_end,{x,0}}.


{function, pwrite, 3, 141}.
  {label,140}.
    {line,[{location,"prim_file.erl",336}]}.
    {func_info,{atom,prim_file},{atom,pwrite},3}.
  {label,141}.
    {allocate,4,3}.
    {init,{y,0}}.
    {move,{x,2},{y,1}}.
    {move,{x,1},{y,2}}.
    {'try',{y,3},{f,142}}.
    {line,[{location,"prim_file.erl",338}]}.
    {call,1,{f,188}}.
    {test,is_map,{f,144},[{x,0}]}.
    {get_map_elements,{f,144},
                      {x,0},
                      {list,[{atom,handle},{x,2},{atom,r_buffer},{x,1}]}}.
    {move,{x,2},{y,0}}.
    {move,{x,1},{x,0}}.
    {line,[{location,"prim_file.erl",339}]}.
    {call_ext,1,{extfunc,prim_buffer,wipe,1}}.
    {move,{y,1},{x,0}}.
    {kill,{y,1}}.
    {line,[{location,"prim_file.erl",340}]}.
    {call_ext,1,{extfunc,erlang,iolist_to_iovec,1}}.
    {move,{y,2},{x,1}}.
    {move,{x,0},{x,2}}.
    {move,{y,0},{x,0}}.
    {kill,{y,0}}.
    {kill,{y,2}}.
    {line,[{location,"prim_file.erl",340}]}.
    {call,3,{f,146}}.
    {try_end,{y,3}}.
    {deallocate,4}.
    return.
  {label,142}.
    {try_case,{y,3}}.
    {test,is_eq_exact,{f,143},[{x,0},{atom,error}]}.
    {test,is_eq_exact,{f,143},[{x,1},{atom,badarg}]}.
    {move,{literal,{error,badarg}},{x,0}}.
    {deallocate,4}.
    return.
  {label,143}.
    {line,[]}.
    {bif,raise,{f,0},[{x,2},{x,1}],{x,0}}.
  {label,144}.
    {line,[{location,"prim_file.erl",338}]}.
    {badmatch,{x,0}}.


{function, pwrite_plain, 3, 146}.
  {label,145}.
    {line,[{location,"prim_file.erl",344}]}.
    {func_info,{atom,prim_file},{atom,pwrite_plain},3}.
  {label,146}.
    {allocate,2,3}.
    {move,{x,1},{y,0}}.
    {move,{x,0},{y,1}}.
    {line,[{location,"prim_file.erl",345}]}.
    {call,3,{f,210}}.
    {test,is_tuple,{f,149},[{x,0}]}.
    {select_tuple_arity,{x,0},{f,151},{list,[2,{f,147},3,{f,148}]}}.
  {label,147}.
    {get_tuple_element,{x,0},0,{x,1}}.
    {test,is_eq_exact,{f,151},[{x,1},{atom,error}]}.
    {jump,{f,150}}.
  {label,148}.
    {get_tuple_element,{x,0},0,{x,1}}.
    {test,is_eq_exact,{f,151},[{x,1},{atom,continue}]}.
    {get_tuple_element,{x,0},1,{x,2}}.
    {get_tuple_element,{x,0},2,{x,3}}.
    {line,[{location,"prim_file.erl",347}]}.
    {gc_bif,'+',{f,0},4,[{y,0},{x,2}],{x,1}}.
    {move,{x,3},{x,2}}.
    {move,{y,1},{x,0}}.
    {call_last,3,{f,146},2}.
  {label,149}.
    {test,is_eq_exact,{f,151},[{x,0},{atom,ok}]}.
  {label,150}.
    {deallocate,2}.
    return.
  {label,151}.
    {line,[{location,"prim_file.erl",345}]}.
    {case_end,{x,0}}.


{function, pwrite, 2, 153}.
  {label,152}.
    {line,[{location,"prim_file.erl",354}]}.
    {func_info,{atom,prim_file},{atom,pwrite},2}.
  {label,153}.
    {allocate,3,2}.
    {init,{y,0}}.
    {move,{x,1},{y,1}}.
    {'try',{y,2},{f,154}}.
    {line,[{location,"prim_file.erl",356}]}.
    {call,1,{f,188}}.
    {test,is_map,{f,156},[{x,0}]}.
    {get_map_elements,{f,156},
                      {x,0},
                      {list,[{atom,handle},{x,2},{atom,r_buffer},{x,1}]}}.
    {move,{x,2},{y,0}}.
    {move,{x,1},{x,0}}.
    {line,[{location,"prim_file.erl",357}]}.
    {call_ext,1,{extfunc,prim_buffer,wipe,1}}.
    {move,{y,1},{x,1}}.
    {move,{integer,0},{x,2}}.
    {move,{y,0},{x,0}}.
    {kill,{y,0}}.
    {kill,{y,1}}.
    {line,[{location,"prim_file.erl",358}]}.
    {call,3,{f,158}}.
    {try_end,{y,2}}.
    {deallocate,3}.
    return.
  {label,154}.
    {try_case,{y,2}}.
    {test,is_eq_exact,{f,155},[{x,0},{atom,error}]}.
    {test,is_eq_exact,{f,155},[{x,1},{atom,badarg}]}.
    {move,{literal,{error,badarg}},{x,0}}.
    {deallocate,3}.
    return.
  {label,155}.
    {line,[]}.
    {bif,raise,{f,0},[{x,2},{x,1}],{x,0}}.
  {label,156}.
    {line,[{location,"prim_file.erl",356}]}.
    {badmatch,{x,0}}.


{function, pwrite_list, 3, 158}.
  {label,157}.
    {line,[{location,"prim_file.erl",369}]}.
    {func_info,{atom,prim_file},{atom,pwrite_list},3}.
  {label,158}.
    {test,is_nonempty_list,{f,160},[{x,1}]}.
    {get_list,{x,1},{x,3},{x,4}}.
    {test,is_tuple,{f,157},[{x,3}]}.
    {test,test_arity,{f,157},[{x,3},2]}.
    {allocate,4,5}.
    {get_tuple_element,{x,3},0,{y,0}}.
    {move,{x,0},{y,2}}.
    {move,{x,2},{y,1}}.
    {move,{x,4},{y,3}}.
    {get_tuple_element,{x,3},1,{x,0}}.
    {line,[{location,"prim_file.erl",372}]}.
    {call_ext,1,{extfunc,erlang,iolist_to_iovec,1}}.
    {move,{y,0},{x,1}}.
    {move,{x,0},{x,2}}.
    {kill,{y,0}}.
    {move,{y,2},{x,0}}.
    {line,[{location,"prim_file.erl",372}]}.
    {call,3,{f,146}}.
    {test,is_tuple,{f,159},[{x,0}]}.
    {test,test_arity,{f,161},[{x,0},2]}.
    {get_tuple_element,{x,0},0,{x,1}}.
    {test,is_eq_exact,{f,161},[{x,1},{atom,error}]}.
    {test_heap,6,1}.
    {get_tuple_element,{x,0},1,{x,2}}.
    {put_tuple,2,{x,1}}.
    {put,{y,1}}.
    {put,{x,2}}.
    {put_tuple,2,{x,0}}.
    {put,{atom,error}}.
    {put,{x,1}}.
    {deallocate,4}.
    return.
  {label,159}.
    {test,is_eq_exact,{f,161},[{x,0},{atom,ok}]}.
    {line,[{location,"prim_file.erl",374}]}.
    {gc_bif,'+',{f,0},0,[{y,1},{integer,1}],{x,2}}.
    {move,{y,3},{x,1}}.
    {move,{y,2},{x,0}}.
    {call_last,3,{f,158},4}.
  {label,160}.
    {test,is_nil,{f,157},[{x,1}]}.
    {move,{atom,ok},{x,0}}.
    return.
  {label,161}.
    {line,[{location,"prim_file.erl",372}]}.
    {case_end,{x,0}}.


{function, sendfile, 8, 163}.
  {label,162}.
    {line,[{location,"prim_file.erl",377}]}.
    {func_info,{atom,prim_file},{atom,sendfile},8}.
  {label,163}.
    {test,is_nil,{f,166},[{x,5}]}.
    {test,is_nil,{f,166},[{x,6}]}.
    {allocate,5,4}.
    {move,{x,1},{y,3}}.
    {move,{x,2},{x,1}}.
    {move,{x,3},{x,2}}.
    {move,{atom,sequential},{x,3}}.
    {move,{x,0},{y,0}}.
    {move,{x,2},{y,1}}.
    {move,{x,1},{y,2}}.
    {'try',{y,4},{f,164}}.
    {line,[{location,"prim_file.erl",390}]}.
    {call,4,{f,88}}.
    {move,{y,0},{x,0}}.
    {kill,{y,0}}.
    {line,[{location,"prim_file.erl",391}]}.
    {call,1,{f,177}}.
    {move,{y,2},{x,2}}.
    {move,{x,0},{x,1}}.
    {move,{y,1},{x,3}}.
    {move,{y,3},{x,0}}.
    {kill,{y,1}}.
    {kill,{y,2}}.
    {kill,{y,3}}.
    {line,[{location,"prim_file.erl",391}]}.
    {call_ext,4,{extfunc,prim_inet,sendfile,4}}.
    {try_end,{y,4}}.
    {deallocate,5}.
    return.
  {label,164}.
    {try_case,{y,4}}.
    {test,is_eq_exact,{f,165},[{x,0},{atom,error}]}.
    {test,is_eq_exact,{f,165},[{x,1},{atom,badarg}]}.
    {move,{literal,{error,badarg}},{x,0}}.
    {deallocate,5}.
    return.
  {label,165}.
    {line,[]}.
    {bif,raise,{f,0},[{x,2},{x,1}],{x,0}}.
  {label,166}.
    {move,{literal,{error,enotsup}},{x,0}}.
    return.


{function, ipread_s32bu_p32bu, 3, 168}.
  {label,167}.
    {line,[{location,"prim_file.erl",412}]}.
    {func_info,{atom,prim_file},{atom,ipread_s32bu_p32bu},3}.
  {label,168}.
    {test,is_atom,{f,169},[{x,2}]}.
    {move,{integer,2147483647},{x,2}}.
    {call_only,3,{f,168}}.
  {label,169}.
    {test,is_integer,{f,172},[{x,1}]}.
    {test,is_integer,{f,172},[{x,2}]}.
    {allocate,3,3}.
    {move,{x,2},{y,0}}.
    {move,{x,1},{y,1}}.
    {'try',{y,2},{f,170}}.
    {line,[{location,"prim_file.erl",417}]}.
    {call,1,{f,188}}.
    {test,is_map,{f,173},[{x,0}]}.
    {get_map_elements,{f,173},{x,0},{list,[{atom,handle},{x,1}]}}.
    {move,{y,0},{x,2}}.
    {move,{x,1},{x,0}}.
    {move,{y,1},{x,1}}.
    {kill,{y,0}}.
    {kill,{y,1}}.
    {line,[{location,"prim_file.erl",418}]}.
    {call,3,{f,175}}.
    {try_end,{y,2}}.
    {deallocate,3}.
    return.
  {label,170}.
    {try_case,{y,2}}.
    {test,is_eq_exact,{f,171},[{x,0},{atom,error}]}.
    {test,is_eq_exact,{f,171},[{x,1},{atom,badarg}]}.
    {move,{literal,{error,badarg}},{x,0}}.
    {deallocate,3}.
    return.
  {label,171}.
    {line,[]}.
    {bif,raise,{f,0},[{x,2},{x,1}],{x,0}}.
  {label,172}.
    {move,{literal,{error,badarg}},{x,0}}.
    return.
  {label,173}.
    {line,[{location,"prim_file.erl",417}]}.
    {badmatch,{x,0}}.


{function, ipread_s32bu_p32bu_nif, 3, 175}.
  {label,174}.
    {line,[{location,"prim_file.erl",424}]}.
    {func_info,{atom,prim_file},{atom,ipread_s32bu_p32bu_nif},3}.
  {label,175}.
    {move,{atom,undef},{x,0}}.
    {line,[{location,"prim_file.erl",425}]}.
    {call_ext_only,1,{extfunc,erlang,nif_error,1}}.


{function, get_handle, 1, 177}.
  {label,176}.
    {line,[{location,"prim_file.erl",432}]}.
    {func_info,{atom,prim_file},{atom,get_handle},1}.
  {label,177}.
    {allocate,1,1}.
    {'try',{y,0},{f,178}}.
    {line,[{location,"prim_file.erl",434}]}.
    {call,1,{f,188}}.
    {test,is_map,{f,180},[{x,0}]}.
    {get_map_elements,{f,180},{x,0},{list,[{atom,handle},{x,1}]}}.
    {move,{x,1},{x,0}}.
    {line,[{location,"prim_file.erl",435}]}.
    {call,1,{f,222}}.
    {try_end,{y,0}}.
    {deallocate,1}.
    return.
  {label,178}.
    {try_case,{y,0}}.
    {test,is_eq_exact,{f,179},[{x,0},{atom,error}]}.
    {test,is_eq_exact,{f,179},[{x,1},{atom,badarg}]}.
    {move,{literal,{error,badarg}},{x,0}}.
    {deallocate,1}.
    return.
  {label,179}.
    {line,[]}.
    {bif,raise,{f,0},[{x,2},{x,1}],{x,0}}.
  {label,180}.
    {line,[{location,"prim_file.erl",434}]}.
    {badmatch,{x,0}}.


{function, reset_write_position, 1, 182}.
  {label,181}.
    {line,[{location,"prim_file.erl",442}]}.
    {func_info,{atom,prim_file},{atom,reset_write_position},1}.
  {label,182}.
    {test,is_tagged_tuple,{f,186},[{x,0},3,{atom,file_descriptor}]}.
    {get_tuple_element,{x,0},2,{x,2}}.
    {test,is_map,{f,185},[{x,2}]}.
    {get_map_elements,{f,185},{x,2},{list,[{atom,r_buffer},{x,1}]}}.
    {allocate,1,2}.
    {move,{x,0},{y,0}}.
    {move,{x,1},{x,0}}.
    {line,[{location,"prim_file.erl",444}]}.
    {call_ext,1,{extfunc,prim_buffer,size,1}}.
    {test,is_lt,{f,183},[{integer,0},{x,0}]}.
    {move,{atom,cur},{x,1}}.
    {move,{y,0},{x,0}}.
    {call_last,2,{f,108},1}.
  {label,183}.
    {test,is_eq_exact,{f,184},[{x,0},{integer,0}]}.
    {move,{atom,ok},{x,0}}.
    {deallocate,1}.
    return.
  {label,184}.
    {line,[{location,"prim_file.erl",444}]}.
    {case_end,{x,0}}.
  {label,185}.
    {line,[{location,"prim_file.erl",443}]}.
    {badmatch,{x,2}}.
  {label,186}.
    {move,{literal,{badrecord,file_descriptor}},{x,0}}.
    {line,[{location,"prim_file.erl",443}]}.
    {call_ext,1,{extfunc,erlang,error,1}}.


{function, get_fd_data, 1, 188}.
  {label,187}.
    {line,[{location,"prim_file.erl",449}]}.
    {func_info,{atom,prim_file},{atom,get_fd_data},1}.
  {label,188}.
    {test,is_tagged_tuple,{f,187},[{x,0},3,{atom,file_descriptor}]}.
    {get_tuple_element,{x,0},2,{x,2}}.
    {test,is_map,{f,190},[{x,2}]}.
    {get_map_elements,{f,190},{x,2},{list,[{atom,owner},{x,0}]}}.
    {bif,self,{f,0},[],{x,1}}.
    {test,is_eq_exact,{f,189},[{x,1},{x,0}]}.
    {move,{x,2},{x,0}}.
    return.
  {label,189}.
    {move,{atom,not_on_controlling_process},{x,0}}.
    {line,[{location,"prim_file.erl",453}]}.
    {call_ext,1,{extfunc,erlang,error,1}}.
  {label,190}.
    {line,[{location,"prim_file.erl",450}]}.
    {badmatch,{x,2}}.


{function, build_fd_data, 2, 192}.
  {label,191}.
    {line,[{location,"prim_file.erl",456}]}.
    {func_info,{atom,prim_file},{atom,build_fd_data},2}.
  {label,192}.
    {allocate,3,2}.
    {bif,self,{f,0},[],{y,1}}.
    {move,{x,0},{y,0}}.
    {move,{x,1},{y,2}}.
    {line,[{location,"prim_file.erl",461}]}.
    {call_ext,0,{extfunc,prim_buffer,new,0}}.
    {line,[{location,"prim_file.erl",458}]}.
    {put_map_assoc,{f,0},
                   {literal,#{}},
                   {x,1},
                   1,
                   {list,[{atom,r_buffer},
                          {x,0},
                          {atom,owner},
                          {y,1},
                          {atom,r_ahead_size},
                          {integer,0},
                          {atom,handle},
                          {y,0}]}}.
    {move,{y,2},{x,0}}.
    {call_last,2,{f,194},3}.


{function, fill_fd_option_map, 2, 194}.
  {label,193}.
    {line,[{location,"prim_file.erl",464}]}.
    {func_info,{atom,prim_file},{atom,fill_fd_option_map},2}.
  {label,194}.
    {test,is_nonempty_list,{f,197},[{x,0}]}.
    {get_list,{x,0},{x,2},{x,3}}.
    {test,is_tuple,{f,195},[{x,2}]}.
    {test,test_arity,{f,196},[{x,2},2]}.
    {get_tuple_element,{x,2},0,{x,4}}.
    {test,is_eq_exact,{f,196},[{x,4},{atom,read_ahead}]}.
    {test,is_map,{f,198},[{x,1}]}.
    {get_tuple_element,{x,2},1,{x,5}}.
    {line,[{location,"prim_file.erl",470}]}.
    {put_map_assoc,{f,0},{x,1},{x,1},6,{list,[{atom,r_ahead_size},{x,5}]}}.
    {move,{x,3},{x,0}}.
    {call_only,2,{f,194}}.
  {label,195}.
    {test,is_eq_exact,{f,196},[{x,2},{atom,read_ahead}]}.
    {test_heap,2,4}.
    {put_list,{literal,{read_ahead,65536}},{x,3},{x,0}}.
    {call_only,2,{f,194}}.
  {label,196}.
    {move,{x,3},{x,0}}.
    {call_only,2,{f,194}}.
  {label,197}.
    {test,is_nil,{f,193},[{x,0}]}.
    {move,{x,1},{x,0}}.
    return.
  {label,198}.
    {test_heap,3,2}.
    {put_tuple,2,{x,0}}.
    {put,{atom,badmap}}.
    {put,{x,1}}.
    {line,[{location,"prim_file.erl",470}]}.
    {call_ext,1,{extfunc,erlang,error,1}}.


{function, open_nif, 2, 200}.
  {label,199}.
    {line,[{location,"prim_file.erl",475}]}.
    {func_info,{atom,prim_file},{atom,open_nif},2}.
  {label,200}.
    {move,{atom,undef},{x,0}}.
    {line,[{location,"prim_file.erl",476}]}.
    {call_ext_only,1,{extfunc,erlang,nif_error,1}}.


{function, close_nif, 1, 202}.
  {label,201}.
    {line,[{location,"prim_file.erl",477}]}.
    {func_info,{atom,prim_file},{atom,close_nif},1}.
  {label,202}.
    {move,{atom,undef},{x,0}}.
    {line,[{location,"prim_file.erl",478}]}.
    {call_ext_only,1,{extfunc,erlang,nif_error,1}}.


{function, read_nif, 2, 204}.
  {label,203}.
    {line,[{location,"prim_file.erl",479}]}.
    {func_info,{atom,prim_file},{atom,read_nif},2}.
  {label,204}.
    {move,{atom,undef},{x,0}}.
    {line,[{location,"prim_file.erl",480}]}.
    {call_ext_only,1,{extfunc,erlang,nif_error,1}}.


{function, write_nif, 2, 206}.
  {label,205}.
    {line,[{location,"prim_file.erl",481}]}.
    {func_info,{atom,prim_file},{atom,write_nif},2}.
  {label,206}.
    {move,{atom,undef},{x,0}}.
    {line,[{location,"prim_file.erl",482}]}.
    {call_ext_only,1,{extfunc,erlang,nif_error,1}}.


{function, pread_nif, 3, 208}.
  {label,207}.
    {line,[{location,"prim_file.erl",483}]}.
    {func_info,{atom,prim_file},{atom,pread_nif},3}.
  {label,208}.
    {move,{atom,undef},{x,0}}.
    {line,[{location,"prim_file.erl",484}]}.
    {call_ext_only,1,{extfunc,erlang,nif_error,1}}.


{function, pwrite_nif, 3, 210}.
  {label,209}.
    {line,[{location,"prim_file.erl",485}]}.
    {func_info,{atom,prim_file},{atom,pwrite_nif},3}.
  {label,210}.
    {move,{atom,undef},{x,0}}.
    {line,[{location,"prim_file.erl",486}]}.
    {call_ext_only,1,{extfunc,erlang,nif_error,1}}.


{function, seek_nif, 3, 212}.
  {label,211}.
    {line,[{location,"prim_file.erl",487}]}.
    {func_info,{atom,prim_file},{atom,seek_nif},3}.
  {label,212}.
    {move,{atom,undef},{x,0}}.
    {line,[{location,"prim_file.erl",488}]}.
    {call_ext_only,1,{extfunc,erlang,nif_error,1}}.


{function, sync_nif, 2, 214}.
  {label,213}.
    {line,[{location,"prim_file.erl",489}]}.
    {func_info,{atom,prim_file},{atom,sync_nif},2}.
  {label,214}.
    {move,{atom,undef},{x,0}}.
    {line,[{location,"prim_file.erl",490}]}.
    {call_ext_only,1,{extfunc,erlang,nif_error,1}}.


{function, advise_nif, 4, 216}.
  {label,215}.
    {line,[{location,"prim_file.erl",491}]}.
    {func_info,{atom,prim_file},{atom,advise_nif},4}.
  {label,216}.
    {move,{atom,undef},{x,0}}.
    {line,[{location,"prim_file.erl",492}]}.
    {call_ext_only,1,{extfunc,erlang,nif_error,1}}.


{function, allocate_nif, 3, 218}.
  {label,217}.
    {line,[{location,"prim_file.erl",493}]}.
    {func_info,{atom,prim_file},{atom,allocate_nif},3}.
  {label,218}.
    {move,{atom,undef},{x,0}}.
    {line,[{location,"prim_file.erl",494}]}.
    {call_ext_only,1,{extfunc,erlang,nif_error,1}}.


{function, truncate_nif, 1, 220}.
  {label,219}.
    {line,[{location,"prim_file.erl",495}]}.
    {func_info,{atom,prim_file},{atom,truncate_nif},1}.
  {label,220}.
    {move,{atom,undef},{x,0}}.
    {line,[{location,"prim_file.erl",496}]}.
    {call_ext_only,1,{extfunc,erlang,nif_error,1}}.


{function, get_handle_nif, 1, 222}.
  {label,221}.
    {line,[{location,"prim_file.erl",497}]}.
    {func_info,{atom,prim_file},{atom,get_handle_nif},1}.
  {label,222}.
    {move,{atom,undef},{x,0}}.
    {line,[{location,"prim_file.erl",498}]}.
    {call_ext_only,1,{extfunc,erlang,nif_error,1}}.


{function, delayed_close_nif, 1, 224}.
  {label,223}.
    {line,[{location,"prim_file.erl",499}]}.
    {func_info,{atom,prim_file},{atom,delayed_close_nif},1}.
  {label,224}.
    {move,{atom,undef},{x,0}}.
    {line,[{location,"prim_file.erl",500}]}.
    {call_ext_only,1,{extfunc,erlang,nif_error,1}}.


{function, read_file, 1, 226}.
  {label,225}.
    {line,[{location,"prim_file.erl",506}]}.
    {func_info,{atom,prim_file},{atom,read_file},1}.
  {label,226}.
    {allocate,1,1}.
    {'try',{y,0},{f,227}}.
    {line,[{location,"prim_file.erl",509}]}.
    {call,1,{f,422}}.
    {line,[{location,"prim_file.erl",509}]}.
    {call,1,{f,230}}.
    {try_end,{y,0}}.
    {deallocate,1}.
    return.
  {label,227}.
    {try_case,{y,0}}.
    {test,is_eq_exact,{f,228},[{x,0},{atom,error}]}.
    {test,is_eq_exact,{f,228},[{x,1},{atom,badarg}]}.
    {move,{literal,{error,badarg}},{x,0}}.
    {deallocate,1}.
    return.
  {label,228}.
    {line,[]}.
    {bif,raise,{f,0},[{x,2},{x,1}],{x,0}}.


{function, read_file_nif, 1, 230}.
  {label,229}.
    {line,[{location,"prim_file.erl",513}]}.
    {func_info,{atom,prim_file},{atom,read_file_nif},1}.
  {label,230}.
    {move,{atom,undef},{x,0}}.
    {line,[{location,"prim_file.erl",514}]}.
    {call_ext_only,1,{extfunc,erlang,nif_error,1}}.


{function, write_file, 2, 232}.
  {label,231}.
    {line,[{location,"prim_file.erl",516}]}.
    {func_info,{atom,prim_file},{atom,write_file},2}.
  {label,232}.
    {move,nil,{x,2}}.
    {call_only,3,{f,234}}.


{function, write_file, 3, 234}.
  {label,233}.
    {line,[{location,"prim_file.erl",518}]}.
    {func_info,{atom,prim_file},{atom,write_file},3}.
  {label,234}.
    {allocate_heap,1,4,3}.
    {move,{x,1},{y,0}}.
    {put_list,{atom,binary},{x,2},{x,2}}.
    {put_list,{atom,write},{x,2},{x,1}}.
    {line,[{location,"prim_file.erl",519}]}.
    {call,2,{f,24}}.
    {test,is_tuple,{f,237},[{x,0}]}.
    {test,test_arity,{f,237},[{x,0},2]}.
    {get_tuple_element,{x,0},0,{x,1}}.
    {get_tuple_element,{x,0},1,{x,2}}.
    {test,is_atom,{f,237},[{x,1}]}.
    {select_val,{x,1},{f,237},{list,[{atom,ok},{f,235},{atom,error},{f,236}]}}.
  {label,235}.
    {move,{y,0},{x,1}}.
    {move,{x,2},{y,0}}.
    {move,{x,2},{x,0}}.
    {line,[{location,"prim_file.erl",521}]}.
    {call,2,{f,72}}.
    {move,{x,0},{x,1}}.
    {move,{y,0},{x,0}}.
    {move,{x,1},{y,0}}.
    {line,[{location,"prim_file.erl",522}]}.
    {call,1,{f,33}}.
    {move,{y,0},{x,0}}.
    {deallocate,1}.
    return.
  {label,236}.
    {deallocate,1}.
    return.
  {label,237}.
    {line,[{location,"prim_file.erl",519}]}.
    {case_end,{x,0}}.


{function, read_link, 1, 239}.
  {label,238}.
    {line,[{location,"prim_file.erl",532}]}.
    {func_info,{atom,prim_file},{atom,read_link},1}.
  {label,239}.
    {move,{atom,false},{x,1}}.
    {call_only,2,{f,243}}.


{function, read_link_all, 1, 241}.
  {label,240}.
    {line,[{location,"prim_file.erl",533}]}.
    {func_info,{atom,prim_file},{atom,read_link_all},1}.
  {label,241}.
    {move,{atom,true},{x,1}}.
    {call_only,2,{f,243}}.


{function, read_link_1, 2, 243}.
  {label,242}.
    {line,[{location,"prim_file.erl",535}]}.
    {func_info,{atom,prim_file},{atom,read_link_1},2}.
  {label,243}.
    {allocate,2,2}.
    {move,{x,1},{y,0}}.
    {'try',{y,1},{f,246}}.
    {line,[{location,"prim_file.erl",536}]}.
    {call,1,{f,422}}.
    {line,[{location,"prim_file.erl",536}]}.
    {call,1,{f,391}}.
    {try_end,{y,1}}.
    {test,is_tuple,{f,248},[{x,0}]}.
    {test,test_arity,{f,248},[{x,0},2]}.
    {get_tuple_element,{x,0},0,{x,1}}.
    {get_tuple_element,{x,0},1,{x,2}}.
    {test,is_atom,{f,248},[{x,1}]}.
    {select_val,{x,1},{f,248},{list,[{atom,ok},{f,244},{atom,error},{f,245}]}}.
  {label,244}.
    {move,{y,0},{x,1}}.
    {move,{x,2},{x,0}}.
    {call_last,2,{f,250},2}.
  {label,245}.
    {deallocate,2}.
    return.
  {label,246}.
    {try_case,{y,1}}.
    {test,is_eq_exact,{f,247},[{x,0},{atom,error}]}.
    {test,is_eq_exact,{f,247},[{x,1},{atom,badarg}]}.
    {move,{literal,{error,badarg}},{x,0}}.
    {deallocate,2}.
    return.
  {label,247}.
    {line,[]}.
    {bif,raise,{f,0},[{x,2},{x,1}],{x,0}}.
  {label,248}.
    {line,[{location,"prim_file.erl",536}]}.
    {try_case_end,{x,0}}.


{function, translate_raw_name, 2, 250}.
  {label,249}.
    {line,[{location,"prim_file.erl",543}]}.
    {func_info,{atom,prim_file},{atom,translate_raw_name},2}.
  {label,250}.
    {allocate,2,2}.
    {move,{x,1},{y,0}}.
    {move,{x,0},{y,1}}.
    {line,[{location,"prim_file.erl",544}]}.
    {call,1,{f,424}}.
    {test,is_list,{f,251},[{x,0}]}.
    {test_heap,3,1}.
    {put_tuple,2,{x,1}}.
    {put,{atom,ok}}.
    {put,{x,0}}.
    {move,{x,1},{x,0}}.
    {deallocate,2}.
    return.
  {label,251}.
    {test,is_tagged_tuple,{f,254},[{x,0},2,{atom,error}]}.
    {select_val,{y,0},
                {f,254},
                {list,[{atom,false},{f,252},{atom,true},{f,253}]}}.
  {label,252}.
    {move,{literal,{error,einval}},{x,0}}.
    {deallocate,2}.
    return.
  {label,253}.
    {test_heap,3,0}.
    {put_tuple,2,{x,0}}.
    {put,{atom,ok}}.
    {put,{y,1}}.
    {deallocate,2}.
    return.
  {label,254}.
    {line,[{location,"prim_file.erl",544}]}.
    {case_end,{x,0}}.


{function, list_dir, 1, 256}.
  {label,255}.
    {line,[{location,"prim_file.erl",550}]}.
    {func_info,{atom,prim_file},{atom,list_dir},1}.
  {label,256}.
    {move,{atom,true},{x,1}}.
    {call_only,2,{f,260}}.


{function, list_dir_all, 1, 258}.
  {label,257}.
    {line,[{location,"prim_file.erl",551}]}.
    {func_info,{atom,prim_file},{atom,list_dir_all},1}.
  {label,258}.
    {move,{atom,false},{x,1}}.
    {call_only,2,{f,260}}.


{function, list_dir_1, 2, 260}.
  {label,259}.
    {line,[{location,"prim_file.erl",553}]}.
    {func_info,{atom,prim_file},{atom,list_dir_1},2}.
  {label,260}.
    {allocate,2,2}.
    {move,{x,1},{y,0}}.
    {'try',{y,1},{f,263}}.
    {line,[{location,"prim_file.erl",554}]}.
    {call,1,{f,422}}.
    {line,[{location,"prim_file.erl",554}]}.
    {call,1,{f,389}}.
    {try_end,{y,1}}.
    {test,is_tuple,{f,265},[{x,0}]}.
    {test,test_arity,{f,265},[{x,0},2]}.
    {get_tuple_element,{x,0},0,{x,1}}.
    {get_tuple_element,{x,0},1,{x,2}}.
    {test,is_atom,{f,265},[{x,1}]}.
    {select_val,{x,1},{f,265},{list,[{atom,ok},{f,261},{atom,error},{f,262}]}}.
  {label,261}.
    {move,{y,0},{x,1}}.
    {move,{x,2},{x,0}}.
    {move,nil,{x,2}}.
    {call_last,3,{f,267},2}.
  {label,262}.
    {deallocate,2}.
    return.
  {label,263}.
    {try_case,{y,1}}.
    {test,is_eq_exact,{f,264},[{x,0},{atom,error}]}.
    {test,is_eq_exact,{f,264},[{x,1},{atom,badarg}]}.
    {move,{literal,{error,badarg}},{x,0}}.
    {deallocate,2}.
    return.
  {label,264}.
    {line,[]}.
    {bif,raise,{f,0},[{x,2},{x,1}],{x,0}}.
  {label,265}.
    {line,[{location,"prim_file.erl",554}]}.
    {try_case_end,{x,0}}.


{function, list_dir_convert, 3, 267}.
  {label,266}.
    {line,[{location,"prim_file.erl",561}]}.
    {func_info,{atom,prim_file},{atom,list_dir_convert},3}.
  {label,267}.
    {test,is_nonempty_list,{f,273},[{x,0}]}.
    {allocate,6,3}.
    {init,{y,0}}.
    {init,{y,1}}.
    {get_list,{x,0},{x,0},{y,4}}.
    {move,{x,2},{y,2}}.
    {move,{x,1},{y,3}}.
    {move,{x,0},{y,5}}.
    {line,[{location,"prim_file.erl",564}]}.
    {call,1,{f,424}}.
    {test,is_list,{f,268},[{x,0}]}.
    {test_heap,2,1}.
    {put_list,{x,0},{y,2},{x,2}}.
    {move,{y,3},{x,1}}.
    {move,{y,4},{x,0}}.
    {call_last,3,{f,267},6}.
  {label,268}.
    {test,is_tagged_tuple,{f,274},[{x,0},2,{atom,error}]}.
    {test,is_eq_exact,{f,269},[{y,3},{atom,false}]}.
    {test_heap,2,0}.
    {put_list,{y,5},{y,2},{x,2}}.
    {move,{y,3},{x,1}}.
    {move,{y,4},{x,0}}.
    {call_last,3,{f,267},6}.
  {label,269}.
    {get_tuple_element,{x,0},1,{x,2}}.
    {test,is_atom,{f,272},[{x,2}]}.
    {select_val,{x,2},
                {f,272},
                {list,[{atom,warning},{f,270},{atom,ignore},{f,271}]}}.
  {label,270}.
    {bif,self,{f,0},[],{y,1}}.
    {line,[{location,"prim_file.erl",580}]}.
    {call_ext,0,{extfunc,erlang,group_leader,0}}.
    {move,{x,0},{y,0}}.
    {move,{atom,microsecond},{x,0}}.
    {line,[{location,"prim_file.erl",581}]}.
    {call_ext,1,{extfunc,erlang,system_time,1}}.
    {line,[{location,"prim_file.erl",579}]}.
    {put_map_assoc,{f,0},
                   {literal,#{}},
                   {x,0},
                   1,
                   {list,[{atom,pid},
                          {y,1},
                          {atom,gl},
                          {y,0},
                          {atom,error_logger},
                          {literal,#{tag => warning_msg}},
                          {atom,time},
                          {x,0}]}}.
    {test_heap,8,1}.
    {put_list,{y,5},nil,{x,2}}.
    {put_tuple,5,{x,1}}.
    {put,{atom,log}}.
    {put,{atom,warning}}.
    {put,{literal,"Non-unicode filename ~p ignored\n"}}.
    {put,{x,2}}.
    {put,{x,0}}.
    {move,{atom,logger},{x,0}}.
    {move,{y,2},{y,5}}.
    {trim,3,3}.
    {line,[{location,"prim_file.erl",578}]}.
    send.
    {move,{y,0},{x,1}}.
    {move,{y,2},{x,2}}.
    {move,{y,1},{x,0}}.
    {call_last,3,{f,267},3}.
  {label,271}.
    {move,{y,3},{x,1}}.
    {move,{y,2},{x,2}}.
    {move,{y,4},{x,0}}.
    {call_last,3,{f,267},6}.
  {label,272}.
    {test_heap,6,0}.
    {put_tuple,2,{x,1}}.
    {put,{atom,no_translation}}.
    {put,{y,5}}.
    {put_tuple,2,{x,0}}.
    {put,{atom,error}}.
    {put,{x,1}}.
    {deallocate,6}.
    return.
  {label,273}.
    {test,is_nil,{f,266},[{x,0}]}.
    {test_heap,3,3}.
    {put_tuple,2,{x,0}}.
    {put,{atom,ok}}.
    {put,{x,2}}.
    return.
  {label,274}.
    {line,[{location,"prim_file.erl",564}]}.
    {case_end,{x,0}}.


{function, read_file_info, 1, 276}.
  {label,275}.
    {line,[{location,"prim_file.erl",588}]}.
    {func_info,{atom,prim_file},{atom,read_file_info},1}.
  {label,276}.
    {move,{atom,local},{x,2}}.
    {move,{integer,1},{x,1}}.
    {call_only,3,{f,284}}.


{function, read_file_info, 2, 278}.
  {label,277}.
    {line,[{location,"prim_file.erl",590}]}.
    {func_info,{atom,prim_file},{atom,read_file_info},2}.
  {label,278}.
    {allocate,1,2}.
    {move,{x,0},{y,0}}.
    {move,{atom,local},{x,2}}.
    {move,{atom,time},{x,0}}.
    {line,[{location,"prim_file.erl",591}]}.
    {call,3,{f,417}}.
    {move,{integer,1},{x,1}}.
    {move,{x,0},{x,2}}.
    {move,{y,0},{x,0}}.
    {call_last,3,{f,284},1}.


{function, read_link_info, 1, 280}.
  {label,279}.
    {line,[{location,"prim_file.erl",593}]}.
    {func_info,{atom,prim_file},{atom,read_link_info},1}.
  {label,280}.
    {move,{atom,local},{x,2}}.
    {move,{integer,0},{x,1}}.
    {call_only,3,{f,284}}.


{function, read_link_info, 2, 282}.
  {label,281}.
    {line,[{location,"prim_file.erl",595}]}.
    {func_info,{atom,prim_file},{atom,read_link_info},2}.
  {label,282}.
    {allocate,1,2}.
    {move,{x,0},{y,0}}.
    {move,{atom,local},{x,2}}.
    {move,{atom,time},{x,0}}.
    {line,[{location,"prim_file.erl",596}]}.
    {call,3,{f,417}}.
    {move,{integer,0},{x,1}}.
    {move,{x,0},{x,2}}.
    {move,{y,0},{x,0}}.
    {call_last,3,{f,284},1}.


{function, read_info_1, 3, 284}.
  {label,283}.
    {line,[{location,"prim_file.erl",598}]}.
    {func_info,{atom,prim_file},{atom,read_info_1},3}.
  {label,284}.
    {allocate,5,3}.
    {init,{y,0}}.
    {init,{y,1}}.
    {move,{x,1},{y,2}}.
    {move,{x,2},{y,3}}.
    {'try',{y,4},{f,287}}.
    {line,[{location,"prim_file.erl",600}]}.
    {call,1,{f,422}}.
    {move,{y,2},{x,1}}.
    {kill,{y,2}}.
    {line,[{location,"prim_file.erl",600}]}.
    {call,2,{f,393}}.
    {test,is_tuple,{f,289},[{x,0}]}.
    {test,test_arity,{f,285},[{x,0},2]}.
    {get_tuple_element,{x,0},0,{x,1}}.
    {test,is_ne_exact,{f,286},[{x,1},{atom,error}]}.
  {label,285}.
    {test,is_tagged_tuple,{f,289},[{x,0},14,{atom,file_info}]}.
    {get_tuple_element,{x,0},4,{y,1}}.
    {get_tuple_element,{x,0},5,{y,0}}.
    {move,{x,0},{y,2}}.
    {move,{y,3},{x,1}}.
    {get_tuple_element,{x,0},6,{x,0}}.
    {line,[{location,"prim_file.erl",604}]}.
    {call,2,{f,429}}.
    {move,{x,0},{x,2}}.
    {move,{y,3},{x,1}}.
    {move,{y,0},{x,0}}.
    {move,{x,2},{y,0}}.
    {line,[{location,"prim_file.erl",605}]}.
    {call,2,{f,429}}.
    {move,{x,0},{x,2}}.
    {move,{y,3},{x,1}}.
    {move,{x,2},{y,3}}.
    {move,{y,1},{x,0}}.
    {kill,{y,1}}.
    {line,[{location,"prim_file.erl",606}]}.
    {call,2,{f,429}}.
    {move,{x,0},{x,3}}.
    {move,{y,2},{x,1}}.
    {move,{y,0},{x,2}}.
    {move,{x,3},{y,2}}.
    {kill,{y,0}}.
    {move,{integer,7},{x,0}}.
    {line,[{location,"prim_file.erl",607}]}.
    {call_ext,3,{extfunc,erlang,setelement,3}}.
    {set_tuple_element,{y,3},{x,0},5}.
    {set_tuple_element,{y,2},{x,0},4}.
    {test_heap,3,1}.
    {put_tuple,2,{x,1}}.
    {put,{atom,ok}}.
    {put,{x,0}}.
    {move,{x,1},{x,0}}.
  {label,286}.
    {try_end,{y,4}}.
    {deallocate,5}.
    return.
  {label,287}.
    {try_case,{y,4}}.
    {test,is_eq_exact,{f,288},[{x,0},{atom,error}]}.
    {move,{literal,{error,badarg}},{x,0}}.
    {deallocate,5}.
    return.
  {label,288}.
    {line,[]}.
    {bif,raise,{f,0},[{x,2},{x,1}],{x,0}}.
  {label,289}.
    {move,{literal,{badrecord,file_info}},{x,0}}.
    {line,[{location,"prim_file.erl",604}]}.
    {call_ext,1,{extfunc,erlang,error,1}}.


{function, write_file_info, 2, 291}.
  {label,290}.
    {line,[{location,"prim_file.erl",615}]}.
    {func_info,{atom,prim_file},{atom,write_file_info},2}.
  {label,291}.
    {move,{atom,local},{x,2}}.
    {call_only,3,{f,295}}.


{function, write_file_info, 3, 293}.
  {label,292}.
    {line,[{location,"prim_file.erl",617}]}.
    {func_info,{atom,prim_file},{atom,write_file_info},3}.
  {label,293}.
    {allocate,2,3}.
    {move,{x,0},{y,1}}.
    {move,{x,1},{y,0}}.
    {move,{x,2},{x,1}}.
    {move,{atom,local},{x,2}}.
    {move,{atom,time},{x,0}}.
    {line,[{location,"prim_file.erl",618}]}.
    {call,3,{f,417}}.
    {move,{y,0},{x,1}}.
    {move,{x,0},{x,2}}.
    {move,{y,1},{x,0}}.
    {call_last,3,{f,295},2}.


{function, write_file_info_1, 3, 295}.
  {label,294}.
    {line,[{location,"prim_file.erl",620}]}.
    {func_info,{atom,prim_file},{atom,write_file_info_1},3}.
  {label,295}.
    {test,is_tagged_tuple,{f,300},[{x,1},14,{atom,file_info}]}.
    {allocate,8,3}.
    {get_tuple_element,{x,1},4,{x,4}}.
    {get_tuple_element,{x,1},5,{y,0}}.
    {get_tuple_element,{x,1},6,{y,2}}.
    {get_tuple_element,{x,1},7,{y,6}}.
    {get_tuple_element,{x,1},12,{y,5}}.
    {get_tuple_element,{x,1},13,{y,4}}.
    {move,{x,0},{y,3}}.
    {move,{x,2},{x,1}}.
    {move,{x,1},{y,1}}.
    {move,{x,4},{x,0}}.
    {'try',{y,7},{f,296}}.
    {line,[{location,"prim_file.erl",632}]}.
    {call,2,{f,321}}.
    {move,{x,0},{x,1}}.
    {move,{y,1},{x,2}}.
    {move,{y,0},{x,0}}.
    {move,{x,1},{y,0}}.
    {line,[{location,"prim_file.erl",633}]}.
    {call,3,{f,327}}.
    {move,{x,0},{x,1}}.
    {move,{y,1},{x,2}}.
    {move,{y,2},{x,0}}.
    {move,{x,1},{y,2}}.
    {kill,{y,1}}.
    {line,[{location,"prim_file.erl",634}]}.
    {call,3,{f,330}}.
    {move,{x,0},{x,1}}.
    {move,{y,3},{x,0}}.
    {move,{x,1},{y,3}}.
    {line,[{location,"prim_file.erl",635}]}.
    {call,1,{f,422}}.
    {move,{y,4},{x,2}}.
    {move,{y,5},{x,1}}.
    {move,{x,0},{y,5}}.
    {kill,{y,4}}.
    {line,[{location,"prim_file.erl",639}]}.
    {call,3,{f,302}}.
    {line,[{location,"prim_file.erl",639}]}.
    {call,1,{f,317}}.
    {move,{y,6},{x,1}}.
    {kill,{y,6}}.
    {move,{y,5},{x,0}}.
    {line,[{location,"prim_file.erl",640}]}.
    {call,2,{f,308}}.
    {line,[{location,"prim_file.erl",640}]}.
    {call,1,{f,317}}.
    {move,{y,2},{x,2}}.
    {move,{y,0},{x,1}}.
    {move,{y,3},{x,3}}.
    {move,{y,5},{x,0}}.
    {kill,{y,0}}.
    {kill,{y,2}}.
    {kill,{y,3}}.
    {kill,{y,5}}.
    {line,[{location,"prim_file.erl",641}]}.
    {call,4,{f,313}}.
    {line,[{location,"prim_file.erl",641}]}.
    {call,1,{f,317}}.
    {try_end,{y,7}}.
    {deallocate,8}.
    return.
  {label,296}.
    {try_case,{y,7}}.
    {test,is_atom,{f,299},[{x,0}]}.
    {select_val,{x,0},
                {f,299},
                {list,[{atom,throw},{f,297},{atom,error},{f,298}]}}.
  {label,297}.
    {test_heap,3,2}.
    {put_tuple,2,{x,0}}.
    {put,{atom,error}}.
    {put,{x,1}}.
    {deallocate,8}.
    return.
  {label,298}.
    {move,{literal,{error,badarg}},{x,0}}.
    {deallocate,8}.
    return.
  {label,299}.
    {line,[]}.
    {bif,raise,{f,0},[{x,2},{x,1}],{x,0}}.
  {label,300}.
    {line,[{location,"prim_file.erl",626}]}.
    {badmatch,{x,1}}.


{function, set_owner, 3, 302}.
  {label,301}.
    {line,[{location,"prim_file.erl",647}]}.
    {func_info,{atom,prim_file},{atom,set_owner},3}.
  {label,302}.
    {test,is_eq_exact,{f,303},[{x,2},{atom,undefined}]}.
    {move,{integer,-1},{x,2}}.
    {call_only,3,{f,302}}.
  {label,303}.
    {test,is_eq_exact,{f,304},[{x,1},{atom,undefined}]}.
    {move,{integer,-1},{x,1}}.
    {call_only,3,{f,302}}.
  {label,304}.
    {call_only,3,{f,306}}.


{function, set_owner_nif, 3, 306}.
  {label,305}.
    {line,[{location,"prim_file.erl",653}]}.
    {func_info,{atom,prim_file},{atom,set_owner_nif},3}.
  {label,306}.
    {move,{atom,undef},{x,0}}.
    {line,[{location,"prim_file.erl",654}]}.
    {call_ext_only,1,{extfunc,erlang,nif_error,1}}.


{function, set_permissions, 2, 308}.
  {label,307}.
    {line,[{location,"prim_file.erl",656}]}.
    {func_info,{atom,prim_file},{atom,set_permissions},2}.
  {label,308}.
    {test,is_eq_exact,{f,309},[{x,1},{atom,undefined}]}.
    {move,{atom,ok},{x,0}}.
    return.
  {label,309}.
    {call_only,2,{f,311}}.


{function, set_permissions_nif, 2, 311}.
  {label,310}.
    {line,[{location,"prim_file.erl",660}]}.
    {func_info,{atom,prim_file},{atom,set_permissions_nif},2}.
  {label,311}.
    {move,{atom,undef},{x,0}}.
    {line,[{location,"prim_file.erl",661}]}.
    {call_ext_only,1,{extfunc,erlang,nif_error,1}}.


{function, set_time, 4, 313}.
  {label,312}.
    {line,[{location,"prim_file.erl",663}]}.
    {func_info,{atom,prim_file},{atom,set_time},4}.
  {label,313}.
    {call_only,4,{f,315}}.


{function, set_time_nif, 4, 315}.
  {label,314}.
    {line,[{location,"prim_file.erl",665}]}.
    {func_info,{atom,prim_file},{atom,set_time_nif},4}.
  {label,315}.
    {move,{atom,undef},{x,0}}.
    {line,[{location,"prim_file.erl",666}]}.
    {call_ext_only,1,{extfunc,erlang,nif_error,1}}.


{function, throw_on_error, 1, 317}.
  {label,316}.
    {line,[{location,"prim_file.erl",668}]}.
    {func_info,{atom,prim_file},{atom,throw_on_error},1}.
  {label,317}.
    {test,is_tuple,{f,318},[{x,0}]}.
    {test,test_arity,{f,316},[{x,0},2]}.
    {get_tuple_element,{x,0},0,{x,1}}.
    {test,is_eq_exact,{f,316},[{x,1},{atom,error}]}.
    {get_tuple_element,{x,0},1,{x,2}}.
    {test,is_eq_exact,{f,319},[{x,2},{atom,enotsup}]}.
    {move,{atom,ok},{x,0}}.
    return.
  {label,318}.
    {test,is_eq_exact,{f,316},[{x,0},{atom,ok}]}.
    return.
  {label,319}.
    {move,{x,2},{x,0}}.
    {line,[{location,"prim_file.erl",670}]}.
    {call_ext,1,{extfunc,erlang,throw,1}}.


{function, file_info_convert_atime, 2, 321}.
  {label,320}.
    {line,[{location,"prim_file.erl",672}]}.
    {func_info,{atom,prim_file},{atom,file_info_convert_atime},2}.
  {label,321}.
    {test,is_ne_exact,{f,322},[{x,0},{atom,undefined}]}.
    {call_only,2,{f,434}}.
  {label,322}.
    {test,is_atom,{f,320},[{x,1}]}.
    {select_val,{x,1},
                {f,320},
                {list,[{atom,posix},
                       {f,323},
                       {atom,universal},
                       {f,324},
                       {atom,local},
                       {f,325}]}}.
  {label,323}.
    {allocate,0,0}.
    {line,[{location,"prim_file.erl",679}]}.
    {call_ext,0,{extfunc,erlang,universaltime,0}}.
    {line,[{location,"prim_file.erl",679}]}.
    {call_ext_last,1,{extfunc,erlang,universaltime_to_posixtime,1},0}.
  {label,324}.
    {allocate,0,0}.
    {line,[{location,"prim_file.erl",677}]}.
    {call_ext,0,{extfunc,erlang,universaltime,0}}.
    {move,{atom,universal},{x,1}}.
    {call_last,2,{f,434},0}.
  {label,325}.
    {allocate,0,0}.
    {line,[{location,"prim_file.erl",675}]}.
    {call_ext,0,{extfunc,erlang,localtime,0}}.
    {move,{atom,local},{x,1}}.
    {call_last,2,{f,434},0}.


{function, file_info_convert_mtime, 3, 327}.
  {label,326}.
    {line,[{location,"prim_file.erl",681}]}.
    {func_info,{atom,prim_file},{atom,file_info_convert_mtime},3}.
  {label,327}.
    {test,is_eq_exact,{f,328},[{x,0},{atom,undefined}]}.
    {move,{x,1},{x,0}}.
    return.
  {label,328}.
    {move,{x,2},{x,1}}.
    {call_only,2,{f,434}}.


{function, file_info_convert_ctime, 3, 330}.
  {label,329}.
    {line,[{location,"prim_file.erl",686}]}.
    {func_info,{atom,prim_file},{atom,file_info_convert_ctime},3}.
  {label,330}.
    {test,is_eq_exact,{f,331},[{x,0},{atom,undefined}]}.
    {move,{x,1},{x,0}}.
    return.
  {label,331}.
    {move,{x,2},{x,1}}.
    {call_only,2,{f,434}}.


{function, get_cwd, 1, 333}.
  {label,332}.
    {line,[{location,"prim_file.erl",693}]}.
    {func_info,{atom,prim_file},{atom,get_cwd},1}.
  {label,333}.
    {test,is_nonempty_list,{f,336},[{x,0}]}.
    {get_list,{x,0},{x,1},{x,2}}.
    {test,is_nonempty_list,{f,335},[{x,2}]}.
    {get_list,{x,2},{x,3},{x,4}}.
    {test,is_eq_exact,{f,335},[{x,3},{integer,58}]}.
    {test,is_nil,{f,335},[{x,4}]}.
    {test,is_ge,{f,335},[{x,1},{integer,65}]}.
    {test,is_ge,{f,334},[{integer,90},{x,1}]}.
    {line,[{location,"prim_file.erl",694}]}.
    {gc_bif,'-',{f,0},2,[{x,1},{integer,65}],{x,0}}.
    {line,[{location,"prim_file.erl",694}]}.
    {gc_bif,'+',{f,0},1,[{x,0},{integer,1}],{x,0}}.
    {call_only,1,{f,338}}.
  {label,334}.
    {test,is_ge,{f,335},[{x,1},{integer,97}]}.
    {test,is_ge,{f,335},[{integer,122},{x,1}]}.
    {line,[{location,"prim_file.erl",696}]}.
    {gc_bif,'-',{f,0},2,[{x,1},{integer,97}],{x,0}}.
    {line,[{location,"prim_file.erl",696}]}.
    {gc_bif,'+',{f,0},1,[{x,0},{integer,1}],{x,0}}.
    {call_only,1,{f,338}}.
  {label,335}.
    {move,{literal,{error,einval}},{x,0}}.
    return.
  {label,336}.
    {move,{literal,{error,badarg}},{x,0}}.
    return.


{function, get_dcwd, 1, 338}.
  {label,337}.
    {line,[{location,"prim_file.erl",701}]}.
    {func_info,{atom,prim_file},{atom,get_dcwd},1}.
  {label,338}.
    {allocate,1,1}.
    {'try',{y,0},{f,341}}.
    {line,[{location,"prim_file.erl",702}]}.
    {call,1,{f,407}}.
    {try_end,{y,0}}.
    {test,is_tuple,{f,343},[{x,0}]}.
    {test,test_arity,{f,343},[{x,0},2]}.
    {get_tuple_element,{x,0},0,{x,1}}.
    {get_tuple_element,{x,0},1,{x,2}}.
    {test,is_atom,{f,343},[{x,1}]}.
    {select_val,{x,1},{f,343},{list,[{atom,ok},{f,339},{atom,error},{f,340}]}}.
  {label,339}.
    {move,{x,2},{x,0}}.
    {line,[{location,"prim_file.erl",703}]}.
    {call,1,{f,424}}.
    {test_heap,3,1}.
    {put_tuple,2,{x,1}}.
    {put,{atom,ok}}.
    {put,{x,0}}.
    {move,{x,1},{x,0}}.
    {deallocate,1}.
    return.
  {label,340}.
    {deallocate,1}.
    return.
  {label,341}.
    {try_case,{y,0}}.
    {test,is_eq_exact,{f,342},[{x,0},{atom,error}]}.
    {test,is_eq_exact,{f,342},[{x,1},{atom,badarg}]}.
    {move,{literal,{error,badarg}},{x,0}}.
    {deallocate,1}.
    return.
  {label,342}.
    {line,[]}.
    {bif,raise,{f,0},[{x,2},{x,1}],{x,0}}.
  {label,343}.
    {line,[{location,"prim_file.erl",702}]}.
    {try_case_end,{x,0}}.


{function, get_cwd, 0, 345}.
  {label,344}.
    {line,[{location,"prim_file.erl",709}]}.
    {func_info,{atom,prim_file},{atom,get_cwd},0}.
  {label,345}.
    {allocate,1,0}.
    {'try',{y,0},{f,348}}.
    {line,[{location,"prim_file.erl",710}]}.
    {call,0,{f,411}}.
    {try_end,{y,0}}.
    {test,is_tuple,{f,350},[{x,0}]}.
    {test,test_arity,{f,350},[{x,0},2]}.
    {get_tuple_element,{x,0},0,{x,1}}.
    {get_tuple_element,{x,0},1,{x,2}}.
    {test,is_atom,{f,350},[{x,1}]}.
    {select_val,{x,1},{f,350},{list,[{atom,ok},{f,346},{atom,error},{f,347}]}}.
  {label,346}.
    {move,{x,2},{x,0}}.
    {line,[{location,"prim_file.erl",711}]}.
    {call,1,{f,424}}.
    {test_heap,3,1}.
    {put_tuple,2,{x,1}}.
    {put,{atom,ok}}.
    {put,{x,0}}.
    {move,{x,1},{x,0}}.
    {deallocate,1}.
    return.
  {label,347}.
    {deallocate,1}.
    return.
  {label,348}.
    {try_case,{y,0}}.
    {test,is_eq_exact,{f,349},[{x,0},{atom,error}]}.
    {test,is_eq_exact,{f,349},[{x,1},{atom,badarg}]}.
    {move,{literal,{error,badarg}},{x,0}}.
    {deallocate,1}.
    return.
  {label,349}.
    {line,[]}.
    {bif,raise,{f,0},[{x,2},{x,1}],{x,0}}.
  {label,350}.
    {line,[{location,"prim_file.erl",710}]}.
    {try_case_end,{x,0}}.


{function, set_cwd, 1, 352}.
  {label,351}.
    {line,[{location,"prim_file.erl",716}]}.
    {func_info,{atom,prim_file},{atom,set_cwd},1}.
  {label,352}.
    {allocate,2,1}.
    {move,{x,0},{y,0}}.
    {'try',{y,1},{f,356}}.
    {line,[{location,"prim_file.erl",718}]}.
    {call,1,{f,426}}.
    {test,is_atom,{f,358},[{x,0}]}.
    {select_val,{x,0},
                {f,358},
                {list,[{atom,true},{f,353},{atom,false},{f,354}]}}.
  {label,353}.
    {move,{y,0},{x,0}}.
    {kill,{y,0}}.
    {line,[{location,"prim_file.erl",719}]}.
    {call,1,{f,422}}.
    {line,[{location,"prim_file.erl",719}]}.
    {call,1,{f,409}}.
    {jump,{f,355}}.
  {label,354}.
    {move,{literal,{error,no_translation}},{x,0}}.
  {label,355}.
    {try_end,{y,1}}.
    {deallocate,2}.
    return.
  {label,356}.
    {try_case,{y,1}}.
    {test,is_eq_exact,{f,357},[{x,0},{atom,error}]}.
    {test,is_eq_exact,{f,357},[{x,1},{atom,badarg}]}.
    {move,{literal,{error,badarg}},{x,0}}.
    {deallocate,2}.
    return.
  {label,357}.
    {line,[]}.
    {bif,raise,{f,0},[{x,2},{x,1}],{x,0}}.
  {label,358}.
    {line,[{location,"prim_file.erl",718}]}.
    {case_end,{x,0}}.


{function, delete, 1, 360}.
  {label,359}.
    {line,[{location,"prim_file.erl",726}]}.
    {func_info,{atom,prim_file},{atom,delete},1}.
  {label,360}.
    {allocate,1,1}.
    {'try',{y,0},{f,361}}.
    {line,[{location,"prim_file.erl",728}]}.
    {call,1,{f,422}}.
    {line,[{location,"prim_file.erl",728}]}.
    {call,1,{f,403}}.
    {try_end,{y,0}}.
    {deallocate,1}.
    return.
  {label,361}.
    {try_case,{y,0}}.
    {test,is_eq_exact,{f,362},[{x,0},{atom,error}]}.
    {test,is_eq_exact,{f,362},[{x,1},{atom,badarg}]}.
    {move,{literal,{error,badarg}},{x,0}}.
    {deallocate,1}.
    return.
  {label,362}.
    {line,[]}.
    {bif,raise,{f,0},[{x,2},{x,1}],{x,0}}.


{function, rename, 2, 364}.
  {label,363}.
    {line,[{location,"prim_file.erl",733}]}.
    {func_info,{atom,prim_file},{atom,rename},2}.
  {label,364}.
    {allocate,2,2}.
    {move,{x,1},{y,0}}.
    {'try',{y,1},{f,365}}.
    {line,[{location,"prim_file.erl",735}]}.
    {call,1,{f,422}}.
    {move,{x,0},{x,1}}.
    {move,{y,0},{x,0}}.
    {move,{x,1},{y,0}}.
    {line,[{location,"prim_file.erl",735}]}.
    {call,1,{f,422}}.
    {move,{x,0},{x,1}}.
    {move,{y,0},{x,0}}.
    {kill,{y,0}}.
    {line,[{location,"prim_file.erl",735}]}.
    {call,2,{f,399}}.
    {try_end,{y,1}}.
    {deallocate,2}.
    return.
  {label,365}.
    {try_case,{y,1}}.
    {test,is_eq_exact,{f,366},[{x,0},{atom,error}]}.
    {test,is_eq_exact,{f,366},[{x,1},{atom,badarg}]}.
    {move,{literal,{error,badarg}},{x,0}}.
    {deallocate,2}.
    return.
  {label,366}.
    {line,[]}.
    {bif,raise,{f,0},[{x,2},{x,1}],{x,0}}.


{function, make_dir, 1, 368}.
  {label,367}.
    {line,[{location,"prim_file.erl",739}]}.
    {func_info,{atom,prim_file},{atom,make_dir},1}.
  {label,368}.
    {allocate,1,1}.
    {'try',{y,0},{f,369}}.
    {line,[{location,"prim_file.erl",741}]}.
    {call,1,{f,422}}.
    {line,[{location,"prim_file.erl",741}]}.
    {call,1,{f,401}}.
    {try_end,{y,0}}.
    {deallocate,1}.
    return.
  {label,369}.
    {try_case,{y,0}}.
    {test,is_eq_exact,{f,370},[{x,0},{atom,error}]}.
    {test,is_eq_exact,{f,370},[{x,1},{atom,badarg}]}.
    {move,{literal,{error,badarg}},{x,0}}.
    {deallocate,1}.
    return.
  {label,370}.
    {line,[]}.
    {bif,raise,{f,0},[{x,2},{x,1}],{x,0}}.


{function, del_dir, 1, 372}.
  {label,371}.
    {line,[{location,"prim_file.erl",745}]}.
    {func_info,{atom,prim_file},{atom,del_dir},1}.
  {label,372}.
    {allocate,1,1}.
    {'try',{y,0},{f,373}}.
    {line,[{location,"prim_file.erl",747}]}.
    {call,1,{f,422}}.
    {line,[{location,"prim_file.erl",747}]}.
    {call,1,{f,405}}.
    {try_end,{y,0}}.
    {deallocate,1}.
    return.
  {label,373}.
    {try_case,{y,0}}.
    {test,is_eq_exact,{f,374},[{x,0},{atom,error}]}.
    {test,is_eq_exact,{f,374},[{x,1},{atom,badarg}]}.
    {move,{literal,{error,badarg}},{x,0}}.
    {deallocate,1}.
    return.
  {label,374}.
    {line,[]}.
    {bif,raise,{f,0},[{x,2},{x,1}],{x,0}}.


{function, make_link, 2, 376}.
  {label,375}.
    {line,[{location,"prim_file.erl",751}]}.
    {func_info,{atom,prim_file},{atom,make_link},2}.
  {label,376}.
    {allocate,2,2}.
    {move,{x,1},{y,0}}.
    {'try',{y,1},{f,377}}.
    {line,[{location,"prim_file.erl",753}]}.
    {call,1,{f,422}}.
    {move,{x,0},{x,1}}.
    {move,{y,0},{x,0}}.
    {move,{x,1},{y,0}}.
    {line,[{location,"prim_file.erl",753}]}.
    {call,1,{f,422}}.
    {move,{x,0},{x,1}}.
    {move,{y,0},{x,0}}.
    {kill,{y,0}}.
    {line,[{location,"prim_file.erl",753}]}.
    {call,2,{f,395}}.
    {try_end,{y,1}}.
    {deallocate,2}.
    return.
  {label,377}.
    {try_case,{y,1}}.
    {test,is_eq_exact,{f,378},[{x,0},{atom,error}]}.
    {test,is_eq_exact,{f,378},[{x,1},{atom,badarg}]}.
    {move,{literal,{error,badarg}},{x,0}}.
    {deallocate,2}.
    return.
  {label,378}.
    {line,[]}.
    {bif,raise,{f,0},[{x,2},{x,1}],{x,0}}.


{function, make_symlink, 2, 380}.
  {label,379}.
    {line,[{location,"prim_file.erl",757}]}.
    {func_info,{atom,prim_file},{atom,make_symlink},2}.
  {label,380}.
    {allocate,2,2}.
    {move,{x,1},{y,0}}.
    {'try',{y,1},{f,381}}.
    {line,[{location,"prim_file.erl",759}]}.
    {call,1,{f,422}}.
    {move,{x,0},{x,1}}.
    {move,{y,0},{x,0}}.
    {move,{x,1},{y,0}}.
    {line,[{location,"prim_file.erl",759}]}.
    {call,1,{f,422}}.
    {move,{x,0},{x,1}}.
    {move,{y,0},{x,0}}.
    {kill,{y,0}}.
    {line,[{location,"prim_file.erl",759}]}.
    {call,2,{f,397}}.
    {try_end,{y,1}}.
    {deallocate,2}.
    return.
  {label,381}.
    {try_case,{y,1}}.
    {test,is_eq_exact,{f,382},[{x,0},{atom,error}]}.
    {test,is_eq_exact,{f,382},[{x,1},{atom,badarg}]}.
    {move,{literal,{error,badarg}},{x,0}}.
    {deallocate,2}.
    return.
  {label,382}.
    {line,[]}.
    {bif,raise,{f,0},[{x,2},{x,1}],{x,0}}.


{function, altname, 1, 384}.
  {label,383}.
    {line,[{location,"prim_file.erl",764}]}.
    {func_info,{atom,prim_file},{atom,altname},1}.
  {label,384}.
    {allocate,1,1}.
    {'try',{y,0},{f,386}}.
    {line,[{location,"prim_file.erl",765}]}.
    {call,1,{f,422}}.
    {line,[{location,"prim_file.erl",765}]}.
    {call,1,{f,413}}.
    {try_end,{y,0}}.
    {test,is_tagged_tuple,{f,385},[{x,0},2,{atom,ok}]}.
    {get_tuple_element,{x,0},1,{x,0}}.
    {line,[{location,"prim_file.erl",766}]}.
    {call,1,{f,424}}.
    {test_heap,3,1}.
    {put_tuple,2,{x,1}}.
    {put,{atom,ok}}.
    {put,{x,0}}.
    {move,{x,1},{x,0}}.
    {deallocate,1}.
    return.
  {label,385}.
    {deallocate,1}.
    return.
  {label,386}.
    {try_case,{y,0}}.
    {test,is_eq_exact,{f,387},[{x,0},{atom,error}]}.
    {test,is_eq_exact,{f,387},[{x,1},{atom,badarg}]}.
    {move,{literal,{error,badarg}},{x,0}}.
    {deallocate,1}.
    return.
  {label,387}.
    {line,[]}.
    {bif,raise,{f,0},[{x,2},{x,1}],{x,0}}.


{function, list_dir_nif, 1, 389}.
  {label,388}.
    {line,[{location,"prim_file.erl",772}]}.
    {func_info,{atom,prim_file},{atom,list_dir_nif},1}.
  {label,389}.
    {move,{atom,undef},{x,0}}.
    {line,[{location,"prim_file.erl",773}]}.
    {call_ext_only,1,{extfunc,erlang,nif_error,1}}.


{function, read_link_nif, 1, 391}.
  {label,390}.
    {line,[{location,"prim_file.erl",774}]}.
    {func_info,{atom,prim_file},{atom,read_link_nif},1}.
  {label,391}.
    {move,{atom,undef},{x,0}}.
    {line,[{location,"prim_file.erl",775}]}.
    {call_ext_only,1,{extfunc,erlang,nif_error,1}}.


{function, read_info_nif, 2, 393}.
  {label,392}.
    {line,[{location,"prim_file.erl",776}]}.
    {func_info,{atom,prim_file},{atom,read_info_nif},2}.
  {label,393}.
    {move,{atom,undef},{x,0}}.
    {line,[{location,"prim_file.erl",777}]}.
    {call_ext_only,1,{extfunc,erlang,nif_error,1}}.


{function, make_hard_link_nif, 2, 395}.
  {label,394}.
    {line,[{location,"prim_file.erl",778}]}.
    {func_info,{atom,prim_file},{atom,make_hard_link_nif},2}.
  {label,395}.
    {move,{atom,undef},{x,0}}.
    {line,[{location,"prim_file.erl",779}]}.
    {call_ext_only,1,{extfunc,erlang,nif_error,1}}.


{function, make_soft_link_nif, 2, 397}.
  {label,396}.
    {line,[{location,"prim_file.erl",780}]}.
    {func_info,{atom,prim_file},{atom,make_soft_link_nif},2}.
  {label,397}.
    {move,{atom,undef},{x,0}}.
    {line,[{location,"prim_file.erl",781}]}.
    {call_ext_only,1,{extfunc,erlang,nif_error,1}}.


{function, rename_nif, 2, 399}.
  {label,398}.
    {line,[{location,"prim_file.erl",782}]}.
    {func_info,{atom,prim_file},{atom,rename_nif},2}.
  {label,399}.
    {move,{atom,undef},{x,0}}.
    {line,[{location,"prim_file.erl",783}]}.
    {call_ext_only,1,{extfunc,erlang,nif_error,1}}.


{function, make_dir_nif, 1, 401}.
  {label,400}.
    {line,[{location,"prim_file.erl",784}]}.
    {func_info,{atom,prim_file},{atom,make_dir_nif},1}.
  {label,401}.
    {move,{atom,undef},{x,0}}.
    {line,[{location,"prim_file.erl",785}]}.
    {call_ext_only,1,{extfunc,erlang,nif_error,1}}.


{function, del_file_nif, 1, 403}.
  {label,402}.
    {line,[{location,"prim_file.erl",786}]}.
    {func_info,{atom,prim_file},{atom,del_file_nif},1}.
  {label,403}.
    {move,{atom,undef},{x,0}}.
    {line,[{location,"prim_file.erl",787}]}.
    {call_ext_only,1,{extfunc,erlang,nif_error,1}}.


{function, del_dir_nif, 1, 405}.
  {label,404}.
    {line,[{location,"prim_file.erl",788}]}.
    {func_info,{atom,prim_file},{atom,del_dir_nif},1}.
  {label,405}.
    {move,{atom,undef},{x,0}}.
    {line,[{location,"prim_file.erl",789}]}.
    {call_ext_only,1,{extfunc,erlang,nif_error,1}}.


{function, get_device_cwd_nif, 1, 407}.
  {label,406}.
    {line,[{location,"prim_file.erl",790}]}.
    {func_info,{atom,prim_file},{atom,get_device_cwd_nif},1}.
  {label,407}.
    {move,{atom,undef},{x,0}}.
    {line,[{location,"prim_file.erl",791}]}.
    {call_ext_only,1,{extfunc,erlang,nif_error,1}}.


{function, set_cwd_nif, 1, 409}.
  {label,408}.
    {line,[{location,"prim_file.erl",792}]}.
    {func_info,{atom,prim_file},{atom,set_cwd_nif},1}.
  {label,409}.
    {move,{atom,undef},{x,0}}.
    {line,[{location,"prim_file.erl",793}]}.
    {call_ext_only,1,{extfunc,erlang,nif_error,1}}.


{function, get_cwd_nif, 0, 411}.
  {label,410}.
    {line,[{location,"prim_file.erl",794}]}.
    {func_info,{atom,prim_file},{atom,get_cwd_nif},0}.
  {label,411}.
    {move,{atom,undef},{x,0}}.
    {line,[{location,"prim_file.erl",795}]}.
    {call_ext_only,1,{extfunc,erlang,nif_error,1}}.


{function, altname_nif, 1, 413}.
  {label,412}.
    {line,[{location,"prim_file.erl",796}]}.
    {func_info,{atom,prim_file},{atom,altname_nif},1}.
  {label,413}.
    {move,{atom,undef},{x,0}}.
    {line,[{location,"prim_file.erl",797}]}.
    {call_ext_only,1,{extfunc,erlang,nif_error,1}}.


{function, reverse_list, 1, 415}.
  {label,414}.
    {line,[{location,"prim_file.erl",805}]}.
    {func_info,{atom,prim_file},{atom,reverse_list},1}.
  {label,415}.
    {move,nil,{x,1}}.
    {line,[{location,"prim_file.erl",805}]}.
    {call_ext_only,2,{extfunc,lists,reverse,2}}.


{function, proplist_get_value, 3, 417}.
  {label,416}.
    {line,[{location,"prim_file.erl",807}]}.
    {func_info,{atom,prim_file},{atom,proplist_get_value},3}.
  {label,417}.
    {test,is_nonempty_list,{f,420},[{x,1}]}.
    {get_list,{x,1},{x,3},{x,4}}.
    {test,is_tuple,{f,418},[{x,3}]}.
    {test,test_arity,{f,418},[{x,3},2]}.
    {get_tuple_element,{x,3},0,{x,5}}.
    {test,is_eq_exact,{f,418},[{x,5},{x,0}]}.
    {get_tuple_element,{x,3},1,{x,0}}.
    return.
  {label,418}.
    {test,is_eq_exact,{f,419},[{x,3},{x,0}]}.
    {move,{atom,true},{x,0}}.
    return.
  {label,419}.
    {move,{x,4},{x,1}}.
    {call_only,3,{f,417}}.
  {label,420}.
    {test,is_nil,{f,416},[{x,1}]}.
    {move,{x,2},{x,0}}.
    return.


{function, encode_path, 1, 422}.
  {label,421}.
    {line,[{location,"prim_file.erl",816}]}.
    {func_info,{atom,prim_file},{atom,encode_path},1}.
  {label,422}.
    {line,[{location,"prim_file.erl",817}]}.
    {call_ext_only,1,{extfunc,prim_file,internal_name2native,1}}.


{function, decode_path, 1, 424}.
  {label,423}.
    {line,[{location,"prim_file.erl",818}]}.
    {func_info,{atom,prim_file},{atom,decode_path},1}.
  {label,424}.
    {test,is_binary,{f,423},[{x,0}]}.
    {line,[{location,"prim_file.erl",819}]}.
    {call_ext_only,1,{extfunc,prim_file,internal_native2name,1}}.


{function, is_path_translatable, 1, 426}.
  {label,425}.
    {line,[{location,"prim_file.erl",821}]}.
    {func_info,{atom,prim_file},{atom,is_path_translatable},1}.
  {label,426}.
    {test,is_list,{f,427},[{x,0}]}.
    {move,{atom,true},{x,0}}.
    return.
  {label,427}.
    {line,[{location,"prim_file.erl",824}]}.
    {call_ext_only,1,{extfunc,prim_file,is_translatable,1}}.


{function, from_posix_seconds, 2, 429}.
  {label,428}.
    {line,[{location,"prim_file.erl",831}]}.
    {func_info,{atom,prim_file},{atom,from_posix_seconds},2}.
  {label,429}.
    {test,is_atom,{f,428},[{x,1}]}.
    {select_val,{x,1},
                {f,428},
                {list,[{atom,local},
                       {f,430},
                       {atom,universal},
                       {f,431},
                       {atom,posix},
                       {f,432}]}}.
  {label,430}.
    {test,is_integer,{f,428},[{x,0}]}.
    {allocate,0,1}.
    {line,[{location,"prim_file.erl",836}]}.
    {call_ext,1,{extfunc,erlang,posixtime_to_universaltime,1}}.
    {line,[{location,"prim_file.erl",836}]}.
    {call_ext_last,1,{extfunc,erlang,universaltime_to_localtime,1},0}.
  {label,431}.
    {test,is_integer,{f,428},[{x,0}]}.
    {line,[{location,"prim_file.erl",834}]}.
    {call_ext_only,1,{extfunc,erlang,posixtime_to_universaltime,1}}.
  {label,432}.
    {test,is_integer,{f,428},[{x,0}]}.
    return.


{function, to_posix_seconds, 2, 434}.
  {label,433}.
    {line,[{location,"prim_file.erl",838}]}.
    {func_info,{atom,prim_file},{atom,to_posix_seconds},2}.
  {label,434}.
    {test,is_eq_exact,{f,435},[{x,1},{atom,posix}]}.
    {test,is_integer,{f,435},[{x,0}]}.
    return.
  {label,435}.
    {test,is_tuple,{f,433},[{x,0}]}.
    {test,test_arity,{f,433},[{x,0},2]}.
    {test,is_atom,{f,433},[{x,1}]}.
    {select_val,{x,1},
                {f,433},
                {list,[{atom,local},{f,436},{atom,universal},{f,437}]}}.
  {label,436}.
    {allocate,0,1}.
    {line,[{location,"prim_file.erl",843}]}.
    {call_ext,1,{extfunc,erlang,localtime_to_universaltime,1}}.
    {line,[{location,"prim_file.erl",843}]}.
    {call_ext_last,1,{extfunc,erlang,universaltime_to_posixtime,1},0}.
  {label,437}.
    {line,[{location,"prim_file.erl",841}]}.
    {call_ext_only,1,{extfunc,erlang,universaltime_to_posixtime,1}}.


{function, module_info, 0, 439}.
  {label,438}.
    {line,[]}.
    {func_info,{atom,prim_file},{atom,module_info},0}.
  {label,439}.
    {move,{atom,prim_file},{x,0}}.
    {line,[]}.
    {call_ext_only,1,{extfunc,erlang,get_module_info,1}}.


{function, module_info, 1, 441}.
  {label,440}.
    {line,[]}.
    {func_info,{atom,prim_file},{atom,module_info},1}.
  {label,441}.
    {move,{x,0},{x,1}}.
    {move,{atom,prim_file},{x,0}}.
    {line,[]}.
    {call_ext_only,2,{extfunc,erlang,get_module_info,2}}.


{function, '-on_load/0-fun-0-', 0, 443}.
  {label,442}.
    {line,[{location,"prim_file.erl",107}]}.
    {func_info,{atom,prim_file},{atom,'-on_load/0-fun-0-'},0}.
  {label,443}.
    {allocate,0,0}.
    {move,{atom,true},{x,1}}.
    {move,{atom,trap_exit},{x,0}}.
    {line,[{location,"prim_file.erl",108}]}.
    {call_ext,2,{extfunc,erlang,process_flag,2}}.
    {call_last,0,{f,10},0}.
