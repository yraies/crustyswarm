#!/usr/bin/bash

linkfoo=$(pwd)

cd result || exit
mkdir /tmp/images
if [[ "$1" == "clean" ]]; then
  rm -r /tmp/images/*
  exit 0
fi
cp ./*/*.png /tmp/images
cd /tmp/images || exit

if [[ ! "$1" == "skip" ]] ; then
  N=16
  (
  for f in $(ls -1 *.png)
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
    conv_"${f//\.png/}".jpg &
  done
  )
fi

mkdir "$linkfoo"/links

for p in $(ls -1 *.png | awk -F '_' '{print $1 "_" $2 "_" $3}' | uniq)
do
  montage conv_"${p}"_*_0100.jpg -tile 6x -geometry x600 mont_"$p".jpg
  ln -fs "$(pwd)"/mont_"$p".jpg "$linkfoo"/links/mont_"$p".jpg
done

for p in $(ls -1 *1000.jpg | awk -F '_' '{print $2 "_" $3 "_" $4}' | uniq)
do
  montage conv_"${p}"_*_1000.jpg -tile 6x -geometry x600 mont_1000_"$p".jpg
  ln -fs "$(pwd)"/mont_1000_"$p".jpg "$linkfoo"/links/mont_1000_"$p".jpg
done

