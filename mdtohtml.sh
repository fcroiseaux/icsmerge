pandoc --css=style.css --metadata title="icsmerge" -V lang=en -V highlighting-css= --mathjax -f markdown+smart \
        --to=html5 -s README.md -o website/index.html
