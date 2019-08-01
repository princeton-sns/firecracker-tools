#/usr/bin/env bash

## Creates an alpine-linux based rootfs for a particular runtime.  ## All
## runtimes share a common prelude and postscript for initialization and are
## specialized by scripts defined in the `runtimes/` subdirectory (typically
## just an `apk` command to install the relevant runtime binaries and libraries).
##
## Usage
## -----
##
## $ ./mk_rtimage.sh RUNTIME OUTPUT_FILE
##
## Where RUNTIME is one of the runtimes defined in `runtimes`, without the `.sh`
## extension, and OUTPUT is the file with the resulting root file system.
##
## Running this script requires super user privileges to mount the target file
## system, but you don't have to run with `sudo`, the script uses `sudo` explicitly.

function print_runtimes() {
  echo -e "Available runtimes:"
  for runtime_file in $(ls runtimes/)
  do
    echo -e "  * $(basename $runtime_file .sh)"
  done
}

## Check command line argument length
if [ "$#" -ne 2 ]; then
  echo "Usage: $0 [RUNTIME] [OUTPUT_FS_IMAGE]"
  print_runtimes
  exit 1
fi

RUNTIME=runtimes/$1.sh
OUTPUT=$2

if [ ! -f "$RUNTIME" ]; then
  echo "Runtime \`$1\` not found."
  print_runtimes
  exit 1
fi

## Create a temporary directory to mount the filesystem
TMPDIR=`mktemp -d`

## Delete the output file if it exists, and create a new one formatted as
## an EXT4 filesystem.
rm -f $OUTPUT
dd if=/dev/zero of=$OUTPUT bs=1M count=500
mkfs.ext4 $OUTPUT

## Mount the output file in the temporary directory
sudo mount $OUTPUT $TMPDIR

## Execute the prelude, runtime script and postscript inside an Alpine docker container
## with the target root file system shared at `/my-rootfs` inside the container.
cat prelude.sh $RUNTIME postscript.sh | docker run -i --rm -v $TMPDIR:/my-rootfs alpine

## Cleanup
sudo umount $TMPDIR
rm -Rf $TMPDIR
