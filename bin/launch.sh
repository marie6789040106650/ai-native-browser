#!/bin/bash
export BROWSER_PATH="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
exec "$(dirname "$0")/core-server" "$@"
