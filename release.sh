# build a new release of resc
# This isn't used for normal compilation (see https://dystroy.org/resc for instruction)
# but for the building of the official releases
version=$(sed 's/version = "\([0-9.]\{1,\}\)"/\1/;t;d' Cargo.toml | head -1)

echo "Building release $version"

# make the build directory and compile for all targets
./compile-all-targets.sh

# add the readme and changelog in the build directory
echo "This is resc. More info and installation instructions on https://github.com/Canop/resc" > build/README.md
cp CHANGELOG.md build

# publish version number
echo "$version" > build/version

# prepare the release archive
rm resc_*.zip
zip -r "resc_$version.zip" build/*

# copy it to releases folder
mkdir releases
cp "resc_$version.zip" releases
