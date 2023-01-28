# Docs

This is specifically for the the following device: Comelit Mini Wi-Fi, version 2.1.0. Doing an `INFO` request for this device should return:

```json
{
  "message":"server-info",
  "message-type":"response",
  "message-id":1,
  "response-code":200,
  "response-string":"OK",
  "model":"MSVF",
  "version":"2.1.0",
  "serial-code":"XXXXXXXX",
  "capabilities":[
    "user-admin-channel",
    "user-auth-channel",
    "configuration-channel",
    "push-notifications-channel",
    "cloudnext-device",
    "fast-activation-channel",
    "cloud-activation",
    "face-recognition-channel"
  ],
  "user-auth-channel":{
    "encryption-required":false
  },
  "user-admin-channel":{
    "encryption-required":false,
    "cloud-code-login":true
  },
  "configuration-channel":{
    "internal-unit-cfg":true,
    "direct-link-cfg":true,
    "iu-buttons-cfg":false,
    "api-version":2
  },
  "fast-activation-channel":{
    "app":true,
    "internal-unit":true,
    "other-device":true
  },
  "cloud-activation":{
    "cloud-activation-enable":false
  }
}
```

## Requests to viper

A couple of facts:
- The device runs at port 61400
- All requests to the viper-server start with `0 6`.

### Header (first 8 bytes)
The first 8 bytes of a viper request
