#!/bin/sh

# If display appears to be steam deck and using native display,
# fix text scaled too high due to bad dpi calculation: https://github.com/rust-windowing/winit/issues/2401

VENDOR=$(cat "/sys/devices/virtual/dmi/id/board_vendor")
echo "Vendor: $VENDOR" > jumpy.log

if [[ $VENDOR == "Valve" ]]; then
  # We are probably a steam deck, check if using native display:
  xrandr >> jumpy.log

  # Identify primary display
  RESOLUTION=$(xrandr | awk '/primary/{print $4}')
  if [ -z $RESOLUTION ]; then
    # If single display / none tagged as primary, use thiss
    RESOLUTION=$(xrandr | awk '/connected/{print $3}')
  fi

  echo "Resolution: $RESOLUTION" >> jumpy.log
  if [[ "$RESOLUTION" =~ '1280x800'* ]]; then
    echo "Launching game with WINIT_X11_SCALE_FACTOR=1 to fix scaling on steam deck display" >> jumpy.log
    WINIT_X11_SCALE_FACTOR=1 ./jumpy 2>&1 >> jumpy.log
    pwd >> jumpy.log

    # Exit if game is closed
    exit 0;
  fi
fi

# Launch game
echo "Launching game without modified scaling" >> jumpy.log
./jumpy 2>&1 >> jumpy.log
