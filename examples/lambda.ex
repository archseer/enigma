defmodule Lambda do

  def start do
    add = fn x -> x + 1 end
    add.(1)
  end

  def start2 do
    sub = captures(2)
    sub.(3)
  end

  def captures(y) do
    sub = fn x -> x - y end
  end
end
