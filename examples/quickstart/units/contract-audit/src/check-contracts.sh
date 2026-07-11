#!/bin/sh
# Enforce dependency contracts (T7b): greeting's post "output starts with 'hello, '",
# announcer's post "uppercase" and "ends with '!'".
set -e
. ../greeting/src/greeting.sh
. ../announcer/src/announcer.sh

g=$(greet sample)
case "$g" in
    "hello, "*) ;;
    *) printf 'not ok 1 - greeting post violated: %s\n' "$g"; exit 1 ;;
esac

a=$(announce sample)
case "$a" in
    *!) ;;
    *) printf 'not ok 1 - announcer post (bang) violated: %s\n' "$a"; exit 1 ;;
esac
[ "$a" = "$(printf '%s' "$a" | tr '[:lower:]' '[:upper:]')" ] || {
    printf 'not ok 1 - announcer post (uppercase) violated: %s\n' "$a"; exit 1
}

printf 'ok 1 - dependency contracts hold\n'
