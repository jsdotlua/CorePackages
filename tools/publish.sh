for f in modules/*; do
    if [ -d "$f" ]; then
        echo "$f"

        wally publish --project-path "$f"
    fi
done
