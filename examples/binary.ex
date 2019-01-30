defmodule Bin do

  def start do
    # a = <<>>
    # a <> str <> "5"
    # add("2")
    parse(<<63, 244, 0, 0, 0, 0, 0, 0>>)
    parse("hello world")
    #interpol("2")
  end

  def parse("hello " <> rest) do
    rest
  end

  def parse(<< float :: float >>) do
    float
  end

  def str do
    "34" 
  end

  def add(x, y) do
    res = x <> "1" 
    q = y + 2
    res <> "3"
  end

  def interpol(x) do
    "asd #{x} q"
  end

  def interpol_types(x, y, z) do
    "asd #{x}" <> <<y :: integer>> <> <<z :: float>>
  end

  def interpol2(x, y) do
    "asd #{x} q" <> "1 #{y + 2}"
  end
end
