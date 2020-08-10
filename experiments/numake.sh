#!/usr/bin/bash

EXPDIR=./result

if [[ "$1" == "clean"  ]]; then
  if [[ -e "$EXPDIR" ]]; then
    echo "Delete all results and configurations?"
    echo "Press Ctrl+C to abort..."
    read
    rm -rf "$EXPDIR"
  else
    echo "Nothing to delete"
  fi
  exit 0
elif [[ "$1" == "subclean"  ]]; then
  echo "Delete all results?"
  echo "Press Ctrl+C to abort..."
  read
  for script in ./result/*.sh
  do
    $script clean
  done
  exit 0
fi

echo "Creating experiment configurations from experiments.json"

filters=$(jq '[path(..|select(type=="array"))] |
               map(select(.[-1]| type != "number")) |
               reduce .[] as $item
               ({}; . + {($item[0]): (.[$item[0]]? + [$item[1:]])})' experiments.json)

mkdir $EXPDIR 2> /dev/null

for base in $(jq -r 'keys | .[]' experiments.json)
do
  echo
  echo "Creating configurations for base configuration: $base"

  all_exist=$(jq --argjson paths "$filters" --arg base "$base" '. as $dot |
    $paths[$base] |
    map(. as $f | $dot | getpath($f) != null)
    | index(false) == null' "base_${base}.json")

  if [[ "$all_exist" == "false" ]]; then
    echo "Problem in $base"
    exit 1
  fi

  pathcount=$(echo "$filters" | jq --arg base "$base" '.[$base] | length')
  for fnr in $(seq  0 $((pathcount-1))); do
    filter=$(jq --argjson paths "$filters" --arg base "$base" --arg fnr "$fnr"\
      '.[$base] | $paths[$base][$fnr | tonumber]' experiments.json)
    pathname=$(jq -r --argjson filter "$filter" --arg base "$base" \
      '.[$base] | getpath($filter)[0]' experiments.json)
    repcount=$(jq -r --argjson filter "$filter" --arg base "$base" \
      '.[$base] | getpath($filter)[2:] | length' experiments.json)

    basename="${base}_$(printf '%02d' ${fnr})_${pathname}"
    echo "#!/usr/bin/bash
if [[ \"\$1\" == \"clean\" ]]; then
  for dir in ./${basename}*/
  do
    rm -r \"\$dir\"
  done
  exit 0
fi" > "${EXPDIR}/${basename}.sh"
    chmod +x "${EXPDIR}/${basename}.sh"

    for rnr in $( seq 2 $(("$repcount"+1)) ); do
      replacement=$(jq -cr --argjson filter "$filter" --arg base "$base" \
        --arg rnr "$rnr" '.[$base] | getpath($filter)[$rnr | tonumber]
        ' experiments.json)
      newargs=$(jq -r --argjson filter "$filter" --arg base "$base" \
        '.[$base] | getpath($filter)[1]' experiments.json)
      filename="${basename}_$(printf '%02d' $((rnr - 2)))_${replacement}.json"

      echo "cargo run ${filename} --no-ui --fixed-camera --orbit-speed 0 --fullscreen --instant $newargs" >> "${EXPDIR}/${basename}.sh"
      echo "Replacing into to $filename"
      jq --argjson filter "$filter" --argjson rep "$replacement" \
        'setpath($filter;$rep)' "base_${base}.json" > "${EXPDIR}/$filename"
    done
  done
done
