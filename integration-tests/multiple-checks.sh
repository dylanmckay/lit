# RUN: sh @file

# CHECK: fizz 1
# CHECK: fizz 2
# CHECK: fizz 100

echo "warning: something is pretty tetchy, bro" 1>&2
echo "warning: ah nah nevermind" 1>&2

for i in $(seq 1 100); do
  echo fizz $i
  echo
done
