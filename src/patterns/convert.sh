#!/bin/bash

rm -f *.rgb
rm -f mod.rs

echo "" >> mod.rs

for i in *.png; do
  name=$(basename $i .png)

  convert $i "$name.rgb"

  echo "pub const IMG_$name: &[u8] = include_bytes!(\"$name.rgb\");" >> mod.rs
done

echo "" >> mod.rs

