#!/bin/sh

manifest=$(pwd)/$1

if [ ! -f $manifest ]; then
  echo "Manifest does not exist: $manifest"
  exit 1
fi

# Extract the version number
version=$(grep -P '"version": "\d+.\d+.\d+"' $manifest | grep -Po "\d+.\d+.\d+")

echo $version

# # Break it down into Major.Minor.Patch
# major=$(echo $version | cut -d. -f1)
# minor=$(echo $version | cut -d. -f2)
# patch=$(echo $version | cut -d. -f3)

# # Increment the patch version
# next_patch=$((patch + 1))

# # Combine them to get the next version
# next_version=$(printf "%d.%d.%d" $major $minor $next_patch)




