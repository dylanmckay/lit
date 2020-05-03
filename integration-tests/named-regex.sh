# RUN: sh @file

# CHECK: hello [[name:\w+]]
echo hello bob

# CHECK: goodbye $$name
echo goodbye bob


