# Pocket Relay Client

![License](https://img.shields.io/github/license/PocketRelay/Client?style=for-the-badge)

## ❔What

This is a tool for Redirecting your local Mass Effect 3 clients to an Unofficial server. The
connection string you put in will be connected to by the client and then used to redirect
traffic using the system hosts file

## ❔Usage Guide

For a guide on using this program see the guide [Here](https://pocket-relay.pages.dev/guide/client/)


## ❔How

This tool uses the system hosts file at `C:/Windows/System32/drivers/etc/hosts` and adds
`127.0.0.1 gosredirector.ea.com` which tells your computer to send all the traffic that
would normally go to `gosredirector.ea.com` to your local computer where its then instead handled by the client tool

## 🔌 Credits

This client application embeds the [https://github.com/Erik-JS/masseffect-binkw32](https://github.com/Erik-JS/masseffect-binkw32) patch DLLs in order to make its redirection work

## EA / BioWare Notice

All code in this repository is authored by Jacobtread and none is taken from BioWare. This code has been 
produced from studying the protocol of the official servers and emulating its functionality. This program is in no way or form supported, endorsed, or provided by BioWare or Electronic Arts.

## 🧾 License

The MIT License (MIT)

Copyright (c) 2022 - 2023 Jacobtread

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
