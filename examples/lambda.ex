defmodule Lambda do

  def start do
    add = fn x -> x + 1 end
    add.(1)
  end

  def captures do
    y = 2
    sub = fn x -> x - y end
  end
end
