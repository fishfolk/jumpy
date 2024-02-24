# If Version is not set, get latest github release.
# example: VERSION=v0.10.0
if [ -z $VERSION ]; then
  VERSION=$(gh release list --json name -L 1 | jq '.[0]'.name | tr -d '"')
fi

# Name of github release, also the root folder name that is inside the downloaded releases.
RELEASE="jumpy-$VERSION"

# If run in root, cd to ./steam
FOLDER="$(basename $(pwd))"
if [ "$FOLDER" != "steam" ]; then
  cd steam
fi

rm -r packaged/*
mkdir -p packaged

cd packaged
APPLE_AMD64="$RELEASE-x86_64-apple-darwin"
gh release download $VERSION --pattern "$APPLE_AMD64.tar.gz"
tar -xzvf  $APPLE_AMD64.tar.gz
cd $RELEASE
rm ../$APPLE_AMD64.tar.gz
zip -r ../$APPLE_AMD64.zip .
cd ..
rm -r $RELEASE

WINDOWS="$RELEASE-x86_64-pc-windows-msvc"
gh release download $VERSION --pattern "$WINDOWS.zip"
unzip $WINDOWS.zip
cd $RELEASE
rm ../$WINDOWS.zip
zip -r ../$WINDOWS.zip .
cd ..
rm -r $RELEASE

LINUX="$RELEASE-x86_64-unknown-linux-gnu"
gh release download $VERSION --pattern "$LINUX.tar.gz"
tar -xzvf  $LINUX.tar.gz
cd $RELEASE
cp ../../launch_scripts/linux/* .
chmod +x
rm ../$LINUX.tar.gz
zip -r ../$LINUX.zip .
cd ..
rm -r $RELEASE

echo ".zip files in ./steam are prepared to be uploaded to steam for respective platforms."
