#!/bin/bash

echo "Sending notifications with different urgency levels..."
echo ""

echo "1. Low urgency (grey border):"
notify-send -u low "Low Urgency" "This is a low priority notification"
sleep 2

echo "2. Normal urgency (pink border - default):"
notify-send -u normal "Normal Urgency" "This is a normal priority notification"
sleep 2

echo "3. Critical urgency (red border):"
notify-send -u critical "Critical Urgency" "This is a critical priority notification!"
sleep 2

