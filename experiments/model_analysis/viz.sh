#!/usr/bin/bash

linkdir=$(pwd)/links
srcdir=$(pwd)/exps
destdir=$(pwd)/procs

CMD=$1
BASE=$2

mkdir -p "$destdir"
mkdir -p "$linkdir"
if [[ "$1" == "clean" ]]; then
  rm -r "$linkdir"/*
  rm -r "$destdir"/*
  exit 0
fi

rm -r "$destdir"/*.png

cp "$srcdir"/*/*.png "$destdir"
cd "$destdir" || exit

GLOBIGNORE='*0000.png:*0000.jpg'

if [[ "$CMD" == "videos" || "$CMD" == "all" ]] ; then
  rm -r "$linkdir"/vid_*"$BASE"* 2>/dev/null
  rm -r "$destdir"/vid_*"$BASE"* 2>/dev/null

  N=16
  (
  for f in $(ls -1 *"$BASE"*0.png)
  do
    ((i=i%N)); ((i++==0)) && wait
    echo "$f"
    text=$(echo "$f" | sed 's/\.png//' | awk -F '_' '{print $1, $2, $4, $6;}')
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

  wait

  for p in $(ls -1 vid_*"$BASE"*.jpg | awk -F '_' '{print $2 "_" $3 "_" $6}' | uniq)
  do
    ffmpeg -r 5 -pattern_type glob -i 'vid_*'"$p"'*100.jpg' -c:v libx264 -y -vf \
    fps=25 -pix_fmt yuv420p -movflags +faststart -tune stillimage -preset slower \
    vid_"$p".mp4
    ln -fs "vid_$(pwd)/$p".mp4 "vid_$linkdir"/"$p".mp4
  done

fi


if [[ "$CMD" == "montages"  || "$CMD" == "all" ]] ; then
  rm -r "$linkdir"/mini_*"$BASE"* 2>/dev/null
  rm -r "$destdir"/mini_*"$BASE"* 2>/dev/null
  rm -r "$linkdir"/mont_*"$BASE"* 2>/dev/null
  rm -r "$destdir"/mont_*"$BASE"* 2>/dev/null

  N=16
  (
  for f in $(ls -1 "$BASE"*00.png)
  do
    ((i=i%N)); ((i++==0)) && wait
    echo "converting $f to mini_${f//\.png/}.jpg"
    text=$(echo "$f" | sed 's/\.png//' | awk -F '_' '{print $4;}')
    convert "$f" \
    -resize 700x700 \
    -font Fira-Mono -pointsize 70 \
    -gravity SouthWest -stroke white -strokewidth 15 -annotate +15+5 "$text" \
    -stroke none -fill black -annotate +15+5 "$text" \
    -quality 90 -format jpg \
    mini_"${f//\.png/}".jpg &
  done
  )
  wait
  sleep 1

  N=4
  (
  for p in $(ls -1 mini_*"$BASE"*00.jpg | awk -F '_' '{print $2 "_" $3 "_" $6 "_" $7}' | sort | uniq)
  do
    ((i=i%N)); ((i++==0)) && wait
    echo "montaging mont_$p"
    glob=$(echo $p | awk -F '_' '{print $1 "_" $2 "*" $3 "_" $4}')
    montage mini_${glob} -tile 5x -geometry x600 -quality 90 mont_"$p"
    ln -fs "$(pwd)"/mont_"$p" "$linkdir"/mont_"$p"
  done
  )
fi

if [[ "$CMD" == "heightmaps"  || "$CMD" == "all" ]] ; then
  rm -r "$linkdir"/hmap_*"$BASE"* 2>/dev/null
  rm -r "$destdir"/hmap_*"$BASE"* 2>/dev/null
  rm -r "$linkdir"/normmap_*"$BASE"* 2>/dev/null
  rm -r "$destdir"/normmap_*"$BASE"* 2>/dev/null
  N=4
  (
  for p in $(ls -1 heightmap_"$BASE"*00.png | awk -F '_' '{print $2 "_" $3 "_" $6 "_" $7}' | sort | uniq)
  do
    ((i=i%N)); ((i++==0)) && wait
    p="${p//\.png/}"
    echo "montaging hmap_${p}.jpg"
    glob=$(echo $p | awk -F '_' '{print $1 "_" $2 "*" $3 "_" $4}')
    montage heightmap_${glob}.png -tile 5x -border 0 -interpolate 'Nearest' -geometry 400x400^ -rotate 180 hmap_"${p}".jpg
    echo "converting hmap_${glob}.jpg to normmap_${p}.jpg"
    convert hmap_"${p}".jpg -contrast-stretch 0x0 normmap_"${p}".jpg
    ln -fs "$(pwd)"/normmap_"$p".jpg "$linkdir"/normmap_"$p".jpg
  done
  )
  wait
fi

if [[ "$CMD" == "all" ]] ; then
  rm -r "$linkdir"/all*.mp4
  rm -r "$destdir"/all*.mp4

  ls -1 *.mp4 | awk '/all/ {print "#file " $0;} !/all/ {print "file " $0 ;}' > concats.txt
  cat concats.txt
  ffmpeg -f concat -i concats.txt -c copy -y all_100_combined.mp4

  ln -fs "$(pwd)"/all_100_combined.mp4 "$linkdir"/all_100_combined.mp4

  ffmpeg -r 5 -pattern_type glob -i 'vid_*.jpg' -c:v libx264 -y -vf \
  fps=25 -pix_fmt yuv420p -movflags +faststart -tune stillimage -preset slower \
  all.mp4

  ln -fs "$(pwd)"/all.mp4 "$linkdir"/all.mp4
fi
