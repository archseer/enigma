defmodule Fib do
  def fib(x) when x == 0, do: 0
  def fib(x) when x == 1, do: 1
  def fib(x), do: fib(x-1) + fib(x-2)
end
