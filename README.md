# Viper Client 👷 (WIP) [![CircleCI](https://dl.circleci.com/status-badge/img/gh/grdw/viper-client/tree/main.svg?style=svg)](https://dl.circleci.com/status-badge/redirect/gh/grdw/viper-client/tree/main)
This is code for my intercom; specifically for the Comelit Mini Wi-Fi MSFV. This features a ViperClient which can talk to the Comelit Mini Wi-fi MSFV directly; so without intermediary Cloud devices.

This code is quite experimental and not ready to be used for anything real. Eventually I'm planning to turn this into a library.

## Setup:

```bash
cp .env.example .env
```

Fill out the details in the `.env` file, and simply type:

```bash
cargo run
```

This will spawn a little demo application which does nothing more than polling the intercom, and executing a bunch of commands.

For a minimal web version check ["viper-web"](/viper-web).

## Missing features/docs:

- [ ] Understanding of the mystical CTPP channel
- [ ] UDP support for being able to watch the camera's
