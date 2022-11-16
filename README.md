# Pocket Relay Client

![License](https://img.shields.io/github/license/PocketRelay/Client?style=for-the-badge)

## ❔What

This is a tool for Redirecting your local Mass Effect 3 clients to an Unofficial server. The
connection string you put in will be connected to by the client and then used to redirect
traffic using the system hosts file

## Connection strings

These are the urls you place in the tool. This url is an HTTP url to the HTTP server running on
the Pocket Relay server. You can ommit the http:// when making the request. Make sure if you 
are a server owner who has changed the port that you include the port in the conneciton string

### Example With Default Port

#### Domain
test.com

> Note: When using domains if the IP changes the client tool will need to be used again
> to update the IP address as the IP address is only resolved at the initial update. This
> is a limitation of the Hosts file

#### Directly use IP address
127.0.0.1

### Example With Custom Port

These examples are for if you are using a custom port for this example
the port is changed to 8080

#### Domain
test.com:8080

#### Directly use IP address
127.0.0.1:8080+

## ❔How

This tool uses the system hosts file at `C:/Windows/System32/drivers/etc/hosts` and adds
`127.0.0.1 gosredirector.ea.com` which tells your computer to send all the traffic that
would normally go to `gosredirector.ea.com` to the host that you provide through the
connection string 

## 🔌 Credits

This client application embeds the [https://github.com/Erik-JS/masseffect-binkw32](https://github.com/Erik-JS/masseffect-binkw32) patch DLLs in order to make its redirection work

## EA / BioWare Notice
All code in this repository is authored by Jacobtread and none is taken from BioWare. This code has been 
produced from studying the protocol of the official servers and emulating its functionality. This program is in no way or form supported, endorsed, or provided by BioWare or Electronic Arts.

## 🧾 License

The MIT License (MIT)

Copyright (c) 2022 Jacobtread

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.ECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
