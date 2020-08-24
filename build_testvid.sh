#/bin/bash
if [ -z "$1" ]; then
  echo "Usage: $0 text"
  exit 1
fi

dir=$(mktemp -d)
trap "rm -rf $dir" EXIT

for i in $(seq 1 60); do
  convert -size 128x128 -gravity center label:"$1" "$dir"/$(printf "%02d" $i).jpg
done

ffmpeg -y -i "$dir"/%02d.jpg -f rawvideo -pix_fmt bgr565be $1.bgr565be
