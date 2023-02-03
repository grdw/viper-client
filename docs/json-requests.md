# JSON requests

## Generic format

```json
{
  "message": "",
  "message-type": "request",
  "message-id": 0
}
```

The `message` is a string which differs per channel:

| Channel  | Message           |
|----------|-------------------|
| UAUT     | access            |
| UCFG     | get-configuration |
| INFO     | server-info       |
| FRCG     | rcg-get-params    |

The `message-id` can be set to anything, it really doesn't matter to the intercom. For the `INFO` and `FRCG` requests, the generic JSON blob will suffice, as long as you pass the correct `message

## UAUT

```json
{
  "message": "access",
  "message-type": "request",
  "message-id": 0,
  "token": "<TOKEN>"
}
```

Replace `<TOKEN>` with your own 32-bit token.

## UCFG

```json
{
  "message": "get-configuration",
  "message-type": "request",
  "message-id": 0,
  "addressbooks": "all"
}
```

There's also an option for `addressbooks` called `"none"` but it's kind of useless knowing that with `"all"` you get a complete response with all the actuators, which are required for opening actual doors.
