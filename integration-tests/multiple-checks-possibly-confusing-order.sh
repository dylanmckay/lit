# RUN: cat @file

echo but this text here is good

# CHECK: hello there
echo hello there

# CHECK: but this text here is good
echo but this text here is good

