#!/bin/sh

# This is a simple example script that prints out an environment variable
#
# In this example, it will print out the RECIPE_MSG variable
#
# This is to show how you can set an env variable in your recipe
# and have it be propagated downward into your scripts

printf "Printing session wide env variables\n"
printf " -> %s: %s\n" "HOME" "$HOME"

printf "\n"

printf "Printing cookbook env variables\n"
printf " -> %s: %s\n" "ASSETS" "$ASSETS"
printf " -> %s: %s\n" "SCRIPTS" "$SCRIPTS"
printf " -> %s: %s\n" "TEMPLATES" "$TEMPLATES"

printf "\n"

printf "Printing recipe specific variables\n"
printf " -> %s: %s\n" "RECIPE_MSG" "$RECIPE_MSG"
