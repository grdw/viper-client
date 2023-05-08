# Viper Client 👷 (WIP) [![CircleCI](https://dl.circleci.com/status-badge/img/gh/grdw/viper-client/tree/main.svg?style=svg)](https://dl.circleci.com/status-badge/redirect/gh/grdw/viper-client/tree/main)
This is code for my intercom; specifically for the Comelit Mini Wi-Fi MSFV. This features a ViperClient which can talk to the Comelit Mini Wi-fi MSFV directly; so without intermediary Cloud devices.

This code is quite experimental and not ready to be used for anything real. Eventually I'm planning to turn this into a library.

## Setup:

To test this library either use ["viper-minimal"](/viper-minimal) or ["viper-web"](/viper-web) to have an interactive demo. The way to set up both these projects:

```bash
cp .env.example .env
```

Fill out the details in the `.env` file, and simply type:

```bash
cargo run
```

This will spawn a little demo application either in your terminal, or a web-server which you can browse to. Please read the respective README's of the demo applications for more information.

## Missing features/docs:

- [ ] Understanding of the mystical CTPP channel
- [ ] UDP support for being able to watch the camera's
