#!/usr/bin/env bash

## Longer running integration tests based on Blargg's GB test suite

fails=0

for test in tests/blargg-gb/cpu_instrs/individual/*.gb
do
  echo Running on $test
  cargo run "$test" TEST >> integration.out 2>> integration.err
  
  if $(cmp screenshots/screenshot.bmp "$test.bmp")
  then
    echo "PASSED $test"
  else
    echo "FAILED $test"
    cp screenshots/screenshot.bmp last_fail.bmp
    fails=$((fails+1))
  fi
  echo ""
done

if [ $fails -gt 0 ]
then
  echo "FAILURE IN $fails tests!"
else
  echo "ALL PASSED"
fi