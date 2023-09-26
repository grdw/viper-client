# Viper-minimal

This little demo application merely polls the doorbell and executes a bunch of functions.

*Setup:*

```bash
cp .env.example .env
```

*To generate a token:*


```bash
cargo run --bin sign_up YOUR-EMAIL-HERE
```

This will return you a `TOKEN` which you need to put into your `.env` file.
