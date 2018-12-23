defmodule Test do

  def start do
    parent = self()
    spawn(Test, :hello, [parent])
    receive do
      {:hello, msg} -> msg
      {:world, msg} -> :no_match #"won't match"
    end
  end

  def hello(parent) do
    # IO.puts "sending"
    send(parent, {:wrong})
    send(parent, {:hello, [1, 2, 3]})
  end
end
