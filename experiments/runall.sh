#!/usr/bin/bash

cd result || exit

for script in ./*.sh
do
  $script
done
