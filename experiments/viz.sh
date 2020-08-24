#!/usr/bin/bash

linkdir=$(pwd)/links
srcdir=$(pwd)/exps
destdir=$(pwd)/procs

mkdir -p "$destdir"
mkdir -p "$linkdir"
if [[ "$1" == "clean" ]]; then
  rm -r "$linkdir"/*
  rm -r "$destdir"/*
  exit 0
fi

cp "$srcdir"/*/*.png "$destdir"
cd "$destdir" || exit

GLOBIGNORE='*0000.png:*0000.jpg'

if [[ "$1" == "videos" || "$1" == "all" ]] ; then
  N=16
  (
  for f in $(ls -1 *0.png)
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

  for p in $(ls -1 vid_*.jpg | awk -F '_' '{print $2 "_" $3}' | uniq)
  do
    ffmpeg -r 5 -pattern_type glob -i 'vid_*'"$p"'*100.jpg' -c:v libx264 -y -vf \
    fps=25 -pix_fmt yuv420p -movflags +faststart -tune stillimage -preset slower \
    "$p".mp4
    ln -fs "$(pwd)/$p".mp4 "$linkdir"/"$p".mp4
  done

  ls -1 *.mp4 | awk '/all/ {print "#file " $0;} !/all/ {print "file " $0 ;}' > concats.txt
  cat concats.txt
  ffmpeg -f concat -i concats.txt -c copy -y all_100_combined.mp4

  ln -fs "$(pwd)"/all_100_combined.mp4 "$linkdir"/all_100_combined.mp4

  ffmpeg -r 5 -pattern_type glob -i 'vid_*.jpg' -c:v libx264 -y -vf \
  fps=25 -pix_fmt yuv420p -movflags +faststart -tune stillimage -preset slower \
  all.mp4

  ln -fs "$(pwd)"/all.mp4 "$linkdir"/all.mp4
fi


if [[ "$1" == "montages"  || "$1" == "all" ]] ; then
  N=16
  (
  for f in $(ls -1 *00.png)
  do
    ((i=i%N)); ((i++==0)) && wait
    echo "converting $f to mini_${f//\.png/}.jpg"
    text=$(echo "$f" | sed 's/\.png//' | awk -F '_' '{print $4;}')
    convert "$f" \
    -resize 1920x1080 \
    -crop 700x700+650+200 +repage \
    -font Fira-Mono -pointsize 45 \
    -gravity SouthWest -stroke white -strokewidth 10 -annotate +10+5 "$text" \
    -stroke none -fill black -annotate +10+5 "$text" \
    -resize 400x400 \
    -quality 90 -format jpg \
    mini_"${f//\.png/}".jpg &
  done
  )
  wait


  for p in $(ls -1 mini_*00.jpg | awk -F '_' '{print $2 "_" $3 "_" $7}' | uniq)
  do
    echo "montaging mont_$p"
    glob=$(echo $p | awk -F '_' '{print $1 "_" $2 "*" $3}')
    montage mini_${glob//\.jpg}.jpg -tile 6x -geometry x400 mont_"$p".jpg
    ln -fs "$(pwd)"/mont_"$p".jpg "$linkdir"/mont_"$p".jpg
  done
fi


