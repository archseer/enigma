-module(fib).
-export([fib/1]).

fib(X) when X == 0 ->
    0;
fib(X) when X == 1 ->
    1;
fib(X) ->
    fib(X-1) + fib(X-2).

% fib(0) -> 0;

% fib(1) -> 1;

% fib(N) -> fib(N-1) + fib(N-2).
