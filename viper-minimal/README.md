# Viper-minimal

This little demo application merely polls the doorbell and executes a bunch of functions.

To start execute:

```bash
cp .env.example .env
```

To generate a token do:


```bash
cargo run --bin sign_up YOUR-EMAIL-HERE
```

This will return you a token which you need to put into your `.env` file.
