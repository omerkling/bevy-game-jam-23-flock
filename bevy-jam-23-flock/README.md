## Run
`cargo run`

## Export for web
(May need to install Trunk tool)

`trunk build`
Then go to `dist` folder and zip the 3 output files to upload to itch.io

## Run local (web)
`cargo run --target wasm32-unknown-unknown --release`
or
`trunk serve --public-url="/"` `
