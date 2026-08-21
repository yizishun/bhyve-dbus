# bhyve-dbus

## Introduction
As the name suggests, bhyve-dbus sits between the socket
interfaces exposed by bhyve (e.g. the console interface) on one side and
exposes a D-Bus server on the other.

Currently the only interface bhyve exposes is the console interface, and only
bhyve's fbuf device exports it (as of writing), letting external programs
connect and render its output.

## Motivation
Because D-Bus is too heavy to be integrated into the
FreeBSD base system, bhyve cannot natively expose a D-Bus interface. To bridge
this gap, we need an out-of-process adapter that communicates with bhyve's
console IPC interface and translates it into the D-Bus display protocol.

This modular design decouples the console IPC from D-Bus, meaning the console
IPC can be adapted to various display protocols simply by swapping out the
adapter. Additionally, it improves security by keeping D-Bus dependencies out
of the core bhyve process.

## Architecture Overview
The bhyve-dbus architecture:

![bhyve-dbus](./bhyve-dbus.drawio.svg)

The relationship between bhyve's fbuf and bhyve-dbus:

![fbuf-bhyve-dbus](./fbuf-bhyve-dbus.drawio.svg)

The D-Bus interface follows QEMU's `org.qemu.Display1` definition (see
[`dbus-display1.xml`](./dbus-display1.xml), copied from QEMU's
`ui/dbus-display1.xml`).
