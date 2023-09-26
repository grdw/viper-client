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
| FACT     | remove-all-users  |
|          | activate-user     |

The `message-id` can be set to anything, it really doesn't matter to the intercom. For the `INFO` and `FRCG` requests, the generic JSON blob will suffice, as long as you pass the correct `message`.

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


## FACT

There are two types of messages which can be executed on the fast activation channel. One is called `remove-all-users` which removes all the old tokens from the device, the other is called `activate-user` which registers a user and returns a token. The request bodies look like:

*remove-all-users*

```json
{
  "message": "remove-all-users",
  "message-type": "request",
  "message-id": 0,
  "requester": "test@test.com"
}
```

*activate-user*

```json
{
  "message": "activate-user",
  "message-type": "request",
  "message-id": 0,
  "email": "test@test.com",
  "description": "a description"
}
```
