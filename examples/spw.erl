-module(spw).
-export([start/0, hello/1]).

start() ->
    spawn(spw, hello, [1]).

hello(x) ->
  x.
