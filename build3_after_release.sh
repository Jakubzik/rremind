#!/bin/bash
#
# ======================================
# EDIT
# PROGRAMMVERSION="0.0.6" 
# MSG="Added functionality to edit config file and archive appointments that are past." # COMMIT MSG FOR GIT
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

# Installiert als crate für cargo install;
# Lädt einen Release by Github hoch
# Lädt PKGBUILD und .SRCINFO (via tools/rremind_upstream/remind) auf AUR
ORDNER="/home/heiko/development/rust/rremind"
AUR_ORDNER="/home/heiko/tools/rremind_upstream"
DATUM=$(date '+%B %d, %Y')
TMPFOLDER="/tmp/rremind"
MANFILE="./rremind.1.gz"

echo "BUILD: Publishing on crates.io?"
cd "$ORDNER"
cargo publish
echo "BUILD: finished"
