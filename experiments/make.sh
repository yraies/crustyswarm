#!/usr/bin/bash

EXPDIR=./result

if [[ "$1" == "clean"  ]]; then
  if [[ -e "$EXPDIR" ]]; then
    rm -rf "$EXPDIR"
  fi
  exit 0
fi

filters=$(jq '[path(..|select(type=="array"))] |
               map(select(.[-1]| type != "number")) |
               reduce .[] as $item
               ({}; . + {($item[0]): (.[$item[0]]? + [$item[1:]])})' experiments.json)

mkdir $EXPDIR 2> /dev/null

for base in $(jq -r 'keys | .[]' experiments.json)
do
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
      '.[$base] | getpath($filter)[1:] | length' experiments.json)

    for rnr in $( seq 1 "$repcount" ); do
      replacement=$(jq -cr --argjson filter "$filter" --arg base "$base" \
        --arg rnr "$rnr" '.[$base] | getpath($filter)[$rnr | tonumber]
        ' experiments.json)
      filename="${EXPDIR}/${base}_${pathname}_${replacement}.json"

      echo "Replacing into to $filename"
      jq --argjson filter "$filter" --argjson rep "$replacement" \
        'setpath($filter;$rep)' "base_${base}.json" >> "$filename"
    done
  done
done
