announce() {
    greet "$1" | tr '[:lower:]' '[:upper:]' | sed 's/$/!/'
}
