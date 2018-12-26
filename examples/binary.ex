defmodule Bin do

  def start do
    add("2")
  end

  def add(x) do
    x <> "1" 
  end

  def interpol(x) do
    "asd #{x} q"
  end
end
