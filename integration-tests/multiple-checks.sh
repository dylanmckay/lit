# RUN: sh @file

# CHECK: hello there
echo hello there

echo there is junk in between the lines

echo right over here

# CHECK: but this text here is good
echo but this text here is good

