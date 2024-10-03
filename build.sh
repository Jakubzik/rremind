#!/bin/bash
#
# ======================================
# EDIT
PROGRAMMVERSION="0.0.1" 
MSG="First working release" # COMMIT MSG FOR GIT
# ======================================

function check_test_outcome {
  ret_code=$1
  if [ "$ret_code" -ne 0 ] ; then
    echo -e "\e[31mFAIL: $ret_code\e[0m";
    exit 1;
  else
    echo -e "\e[32mok\e[0m"
  fi;
}

ORDNER="/home/heiko/development/rust/rremind"
AUR_ORDNER="/home/heiko/tools/rremind_upstream"
DATUM=$(date '+%B %d, %Y')
TMPFOLDER="/tmp/rremind"
MANFILE="./rremind.1.gz"

echo "BUILD: Setting dates and version (to $DATUM and $PROGRAMMVERSION)..."
sed -e "s/#PROGRAMMVERSION#/$PROGRAMMVERSION/g" "$ORDNER/manpage_template.md" > "$ORDNER/manpage.md"
sed -i -e "s/#DATUM#/$DATUM/g" "$ORDNER/manpage.md" 
sed -e "s/#PROGRAMMVERSION#/$PROGRAMMVERSION/g" "$ORDNER/Cargo_template.toml" > "$ORDNER/Cargo.toml"
echo "BUILD: ...set."
echo ""
echo "BUILD: Compiling manpage..."
rm $MANFILE
pandoc ./manpage.md -s -t man -o ./rremind.1
gzip ./rremind.1
check_test_outcome
echo "BUILD: ...compiled"
echo 
echo
echo
sleep 3

echo "BUILD: Compiling binary for AUR with -m..."
cargo-aur -m b
check_test_outcome
echo "BUILD: ...compiled."
echo 
echo
echo
sleep 3

PGV="rremind-$PROGRAMMVERSION-x86_64.tar.gz"
echo "BUILD: Producing binary $PGV with manpage inside..."
rm -rf $TMPFOLDER
mkdir -p "$TMPFOLDER"
cp "$ORDNER/target/cargo-aur/$PGV" "$TMPFOLDER/"
cp "$ORDNER/target/cargo-aur/LICENSE.md" "$TMPFOLDER/"
cp "$ORDNER/rremind.1.gz" "$TMPFOLDER/"
cd $TMPFOLDER
tar -xf "$PGV"
rm "$PGV"
tar -czf $PGV rremind LICENSE.md rremind.1.gz
cp $PGV "$ORDNER/target/cargo-aur/"
cp $PGV ~/tools/rremind_upstream/
echo "BUILD: ...produced."
echo 
echo
echo
sleep 3

cd "$ORDNER"
echo "BUILD: Updating GIT..."
git add .
check_test_outcome
git commit -m "$MSG"
check_test_outcome
git push origin
check_test_outcome
# gh release create v"$PROGRAMMVERSION" --notes "$MSG" "$ORDNER/target/cargo-aur/$PGV"
echo "BUILD: ...committed"
echo 
echo
echo
sleep 3

SHASUM=$(sha256sum  "$AUR_ORDNER/$PGV" | awk '{print $1}')

sed -e "s/#SHASUM#/$SHASUM/g" "$AUR_ORDNER/SRCINFO_template.md" > "$AUR_ORDNER/rremind/.SRCINFO"
sed -i -e "s/#PROGRAMMVERSION#/$PROGRAMMVERSION/g" "$AUR_ORDNER/rremind/.SRCINFO" 

sed -e "s/#SHASUM#/$SHASUM/g" "$AUR_ORDNER/PKGBUILD_template.md" > "$AUR_ORDNER/rremind/PKGBUILD"
sed -i -e "s/#PROGRAMMVERSION#/$PROGRAMMVERSION/g" "$AUR_ORDNER/rremind/PKGBUILD" 

echo "BUILD: Going to $AUR_ORDNER, pushing commit there..."
cd "$AUR_ORDNER/rremind"
git add .
check_test_outcome
git commit -m "$MSG"
check_test_outcome
git push
check_test_outcome
echo "BUILD: ...pushed"
echo 
echo
echo
sleep 3

gh release create v"$PROGRAMMVERSION" "$ORDNER/target/cargo-aur/$PGV"
check_test_outcome
#

echo "BUILD: Publishing on crates.io?"
cd "$ORDNER"
cargo publish
echo "BUILD: finished"
