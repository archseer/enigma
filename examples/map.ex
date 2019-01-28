defmodule Maps do

  def start do
    # a = <<>>
    # a <> str <> "5"
    # add("2")
    a = m()
    a = %{a | b: 1}
    match(a)
  end

  def m do
    %{a: 1}
  end
  def match(%{a: a, b: b}) do
    a + b
  end
end
