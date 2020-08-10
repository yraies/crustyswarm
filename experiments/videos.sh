#!/usr/bin/bash

linkfoo=$(pwd)

cd /tmp/images || exit

for p in $(ls -1 *.png | awk -F '_' '{print $1 "_" $2 "_" $3}' | uniq)
do
  ffmpeg -r 5 -pattern_type glob -i '*'"$p"'*100.jpg' -c:v libx264 -y -vf \
  fps=25 -pix_fmt yuv420p -movflags +faststart -tune stillimage -preset slower \
  "$p".mp4
  ln -fs "$(pwd)/$p".mp4 "$linkfoo"/links/"$p".mp4
done

ls -1 *.mp4 | awk '/all/ {print "#file " $0;} !/all/ {print "file " $0 ;}' > concats.txt
cat concats.txt
ffmpeg -f concat -i concats.txt -c copy -y all_100_combined.mp4

ln -fs "$(pwd)"/all_100_combined.mp4 "$linkfoo"/links/all_100_combined.mp4

ffmpeg -r 5 -pattern_type glob -i '*.jpg' -c:v libx264 -y -vf \
fps=25 -pix_fmt yuv420p -movflags +faststart -tune stillimage -preset slower \
all.mp4

ln -fs "$(pwd)"/all.mp4 "$linkfoo"/links/all.mp4
