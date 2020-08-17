#!/usr/bin/bash

linkfoo=$(pwd)

cd result || exit
mkdir /tmp/images
if [[ "$1" == "clean" ]]; then
  rm -r /tmp/images/*
  rm -r ./links
  exit 0
fi
cp ./*/*.png /tmp/images
mkdir links
cd /tmp/images || exit

if [[ "$1" == "videos" ]] ; then
  N=16
  (
  for f in $(ls -1 *0.png)
  do
    ((i=i%N)); ((i++==0)) && wait
    echo "$f"
    text=$(echo "$f" | sed 's/\.png//' | awk -F '_' '{print $1, $3, $5, $7;}')
    convert "$f" \
    -resize 1920x1080 \
    -crop 700x700+650+200 +repage \
    -font Fira-Mono -pointsize 20 \
    -gravity SouthWest -stroke white -strokewidth 5 -annotate +10+5 "$text" \
    -stroke none -fill black -annotate +10+5 "$text" \
    -quality 95 -format jpg \
    vid_"${f//\.png/}".jpg &
  done
  )

  for p in $(ls -1 vid_*.jpg | awk -F '_' '{print $2 "_" $3 "_" $4}' | uniq)
  do
    ffmpeg -r 5 -pattern_type glob -i 'vid_*'"$p"'*100.jpg' -c:v libx264 -y -vf \
    fps=25 -pix_fmt yuv420p -movflags +faststart -tune stillimage -preset slower \
    "$p".mp4
    ln -fs "$(pwd)/$p".mp4 "$linkfoo"/links/"$p".mp4
  done

  ls -1 *.mp4 | awk '/all/ {print "#file " $0;} !/all/ {print "file " $0 ;}' > concats.txt
  cat concats.txt
  ffmpeg -f concat -i concats.txt -c copy -y all_100_combined.mp4

  ln -fs "$(pwd)"/all_100_combined.mp4 "$linkfoo"/links/all_100_combined.mp4

  ffmpeg -r 5 -pattern_type glob -i 'vid_*.jpg' -c:v libx264 -y -vf \
  fps=25 -pix_fmt yuv420p -movflags +faststart -tune stillimage -preset slower \
  all.mp4

  ln -fs "$(pwd)"/all.mp4 "$linkfoo"/links/all.mp4

elif [[ "$1" == "montages" ]] ; then
  N=16
  (
  for f in $(ls -1 *{1000,100}.png)
  do
    ((i=i%N)); ((i++==0)) && wait
    echo "$f"
    convert "$f" \
    -resize 1920x1080 \
    -crop 700x700+650+200 +repage \
    -resize 400x400 \
    -quality 90 -format jpg \
    mini_"${f//\.png/}".jpg &
  done
  )

  for p in $(ls -1 mini_*_0100.jpg | awk -F '_' '{print $2 "_" $3 "_" $4}' | uniq)
  do
    montage mini_"${p}"_*_0100.jpg -tile 6x -geometry x400 mont_"$p"_0100.jpg
    ln -fs "$(pwd)"/mont_"$p"_0100.jpg "$linkfoo"/links/mont_"$p"_0100.jpg
  done

  for p in $(ls -1 mini_*_1000.jpg | awk -F '_' '{print $2 "_" $3 "_" $4}' | uniq)
  do
    montage mini_"${p}"_*_1000.jpg -tile 6x -geometry x400 mont_"$p"_1000.jpg
    ln -fs "$(pwd)"/mont_"$p"_1000.jpg "$linkfoo"/links/mont_1000_"$p"_1000.jpg
  done
fi




