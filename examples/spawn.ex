defmodule Test do

  def start do
    spawn(Test, :hello, [1])
  end

  def hello(x) do
    x
  end
end
