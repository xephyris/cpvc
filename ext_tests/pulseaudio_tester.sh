# ==============================================================================
# DISCLOSURE, SEPARATION & LICENSING NOTICE
# ==============================================================================
# This file was created with assistance from AI tools. 
# 
# SEPARATION OF CODEBASE: This file is NOT part of the primary human-authored 
# codebase of this project. It is written in a separate programming language 
# and exists solely within an isolated testing environment to serve as an 
# external verification source to ensure cpvc functions correctly.
#
# LICENSE EXCLUSION: This file is strictly EXCLUDED from the GNU General Public 
# License (GPL) governing the rest of this repository. 
#
# PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED.
# ==============================================================================

#!/usr/bin/env bash

set -e

show_help() {
    echo "PulseAudio Volume Control Script"
    echo "================================"
    echo "Usage: $0 [options]"
    echo ""
    echo "Options:"
    echo "  --list-sinks             List all available sinks with their IDs and current volume"
    echo "  --get-vol <id>           Get the volume percentage of a specific sink ID"
    echo "  --set-vol <id> <value>   Set the volume of a specific sink ID (e.g. 0.1, 0.45, ...)"
    echo "  --set-mute <id>          Toggle mute status for a specific sink ID"
    echo "  --get-mute <id>          Get the mute status for a specific sink ID"
    echo ""
    echo "Examples:"
    echo "  $0 --list-sinks"
    echo "  $0 --get-vol 1"
    echo "  $0 --set-vol 1 0.5"
    echo "  $0 --set-mute 1"
    echo "  $0 --get-mute 1"
    exit 1
}

list_sinks() {
    pactl list sinks | awk '
        /Name:/ { print $2 }
    '
}

if [ -z "$1" ]; then
    show_help
fi

COMMAND="$1"
SINK_ID="$2"
VALUE="$3"

if [ -z "$SINK_ID"]; then 
    SINK_ID=$(pactl get-default-sink)
fi

case "$COMMAND" in
    --list-sinks)
        list_sinks
        ;;
    --get-vol)
        if [ -z "$SINK_ID" ]; then
            echo "Error: Missing sink ID."
            show_help
        fi
        VOLUME=$(pactl get-sink-volume "$SINK_ID" | grep -Po '\d+(?=%)' | head -n 1)
        if [ -z "$VOLUME" ]; then
            echo "Error: Could not retrieve volume for Sink ID '$SINK_ID'. Check if the ID is valid." >&2
            exit 1
        fi
        echo "0.${VOLUME}"
        ;;
    --set-vol)
        if [ -z "$SINK_ID" ] || [ -z "$VALUE" ]; then
            echo "Error: Missing sink ID or volume value."
            show_help
        fi

        if [[ ! "$VALUE" =~ %$ ]]; then
            if [[ "$VALUE" =~ ^0\. ]]; then
                VALUE=$(echo "$VALUE * 100" | bc | cut -d. -f1)
            fi
            VALUE="${VALUE}%"
        fi

        pactl set-sink-volume "$SINK_ID" "$VALUE"
        echo "Sink $SINK_ID volume set to $VALUE"
        ;;
    --set-mute)
        if [ -z "$SINK_ID" ]; then
            echo "Error: Missing sink ID."
            show_help
        fi
        pactl set-sink-mute "$SINK_ID" toggle
        MUTE_STATUS=$(pactl get-sink-mute "$SINK_ID" | awk '{print $2}')
        echo "Sink $SINK_ID mute status: $MUTE_STATUS"
        ;;
    --get-mute) 
        if [ -z "$SINK_ID" ]; then
            echo "Error: Missing sink ID."
            show_help
        fi

        MUTE_STATUS=$(pactl get-sink-mute "$SINK_ID" 2>/dev/null | awk '{print $2}')

        if [ -z "$MUTE_STATUS" ]; then
            echo "Error: Could not retrieve mute status. Check if Sink ID '$SINK_ID' is valid." >&2
            exit 1
        fi

        echo "Sink $SINK_ID mute status: $MUTE_STATUS"
        ;;
    --help|-h)
        show_help
        ;;
    *)
        echo "Error: Unknown option '$COMMAND'"
        show_help
        ;;
esac
