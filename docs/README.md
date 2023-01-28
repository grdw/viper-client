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

A couple of facts:
- The device runs at port 61400
- All requests to the viper-server start with `0 6`.
- It speaks a mix of TCP/UDP.

# TCP Requests
## Header format (first 8 bytes)
The first 8 bytes of a viper response and request have the following format:

```
00 06 L1 L2 C1 C2 00 00
```

- The L1 and L2 bytes indicate the length of the bytes to read.
- The C1 and C2 bytes are channel (or control bytes). These are `00 00` in case of some requests.

## Determining the length of the buffer

The length is determined by doing the following:

```
(L1 to decimal) + ((L2 to decimal) * 255) + L2 to decimal
```

## Opening a channel
A channel is opened by executing the following bytes:

```
00 06 0f 00 00 00 00 00  <--- The header
cd ab 01 00 07 00 00 00  <--- A magical constant 8 bytes (this is the same for opening any channel)
55 41 55 54 76 5f 00     <--- The channel to open + 2 control bytes, ending on a zero
``` 

In this particular example I'm opening a `UAUT` (55 41 55 54) channel with the control bytes 76 5F.

In some cases extra data can be passed to opening a channel, which is done like such:

```
00 06 1e 00 00 00 00 00 <--- The header
cd ab 01 00 07 00 00 00 <--- Magic constant
43 54 50 50 7b 5f 00 00 <--- The channel to open + 2 control bytes, ending on two zero's
0a 00 00 00 53 42 30 30 <--- [0a 00 00 00] + the argument, ending on a zero
30 30 30 36 32 00
```

I believe the only example is CTPP where this actually happens.

## Executing a request on a channel
To execute a request on a channel, you have to make sure that the control bytes match. If they do not, viper-server will fail. Also, if the length doesn't match in the request, viper-server will return a bad request. 

An example:

```
00 06 46 00 79 5f 00 00 <-- Header (46 indicates that there are 46 characters submitted after the header), 79 5F are the control bytes
7b 22 6d 65 73 73 61 67 <-- Start of the JSON blob in this case
65 22 3a 22 72 63 67 2d
67 65 74 2d 70 61 72 61 
6d 73 22 2c 22 6d 65 73
73 61 67 65 2d 74 79 70 
65 22 3a 22 72 65 71 75
65 73 74 22 2c 22 6d 65 
73 73 61 67 65 2d 69 64
22 3a 31 32 31 7d       <-- End of the JSON blob
```

The requests can be either JSON or some other syntax of bytes which I've yet to decipher.

## Parsing responses
ILB

# UDP Requests
ILB
