#!/bin/bash

screenshotdir=./screenshots
destdir=./montages

mkdir -p "$destdir"
mkdir -p "$screenshotdir"

rm -r "$screenshotdir"/*.png
rm -r "$destdir"/*.png

GLOBIGNORE='*0000.png:*0000.jpg'

for f in $(ls -1 *.grammar.json)
do
  /usr/local/bin/cargo run --bin viz -- --grammar "$f" --fixed-camera --screenshot-once "$screenshotdir" --no-ui --square
done

mv *.png $screenshotdir

imagesize="640x480"

N=16
(
for f in $(ls -1 "$screenshotdir"/*.png)
do
  ((i=i%N)); ((i++==0)) && wait
  echo "converting $f"
  name=$(echo "$f" | sed "s#$screenshotdir/##" | sed "s/.grammar.json.png//" | sed "s/T[a-z]*/T/g" | awk -F '_' '{OFS = "_"} {print $1, $3, $2;}')
  if [[ $f =~ "mark" ]]; then
    convert "$f" -resize $imagesize -quality 90 -format jpg -bordercolor red -border 4x4 "$screenshotdir"/mini_"$name".jpg &
  else
    convert "$f" -resize $imagesize -quality 90 -format jpg -bordercolor white -border 4x4 "$screenshotdir"/mini_"$name".jpg &
  fi
done
)
wait
sleep 1

N=4
(
for p in $(ls -1 "$screenshotdir"/mini_*.jpg | sed "s#$screenshotdir/##" | awk -F '_' '{print $2 "_" $3 "_" }' | sort | uniq)
do
  ((i=i%N)); ((i++==0)) && wait
  echo "montaging mont_$p"
  montage -label "${p/_/ }" "$screenshotdir"/mini_"$p"* -mode concatenate -tile x1 -geometry +4+8  -quality 90 "$destdir"/mont_"$p".jpg
done
)

montage "$destdir"/mont_* -background black -mode concatenate -geometry +0+0 -tile 1x -quality 80 "$destdir"/all.jpg
