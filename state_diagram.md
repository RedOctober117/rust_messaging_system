# Client-Server State Map

All communication acorss states must be in the form of valid packets containing a `source`, `destination`, `data` field, and `timestamp`.

## Field Layout

Fields should be formed as:

`destination`, `source`, `timestamp`, `data`

Complete messages are prepended by a `VarUInt` denoting the packets length, excluding the size of the `VarUInt` itself.

> `destination` is specified first to limit the processing needing to be done by the server. If the destination is not `0, 0`, then the server simply needs to forward the packet on.

### Source

`Source` consists of the source length as a `VarUInt`, followed by the source name, ie:

`10, TestSource`

If `0` is provided for both components, the server is the intended source.

### Destination

`Destination` is identical to `Source`.

### Data

`Data` begins with the type of data it contains, based upon the state of the client-server connection. Because `data` is the last field of the packet, we can avoid needing to prepend a `VarUInt` length, as it's safe to assume the remainder of the total packet length contains only the `data` field bytes.

### Timestamp

The `timestamp` is formed as a `VarUInt`.

## Vaild States

- Login
- Communication

### Login, 0x00

`Login` data falls under two categories:

- `request`
- `response`

Few packets are exchanged in this state.

#### Client -> Server

- `request_connect`, 0x00
- `response_connect_ack`, 0x01

#### Server -> Client

- `response_connect_success`, 0x00
- `response_connect_fail`, 0x01

#### Handshake

Client and Server begin in `Login, 0x00` state.

1. C -> S: `request_connect` `(0x00)`
2. S -> C: `response_connect_success` `(0x00)`/`response_connect_fail` `(0x01)`
3. C -> S: `response_connect_ack` `(0x01)`

Client and Server transition to `Communication, 0x01` state.

### Communication, 0x01

`Communication` data falls under four general categories:

- `request`
- `response`
- `order`
- `communication`

The former two categories expect some type of return message from the destination (ping request/response, disconnect request/response). The latter do not expect a return message.

`Order` data originate from the server, as a curtesy to the client, to notify it of some event (disconnects, etc.).

#### Client -> Server

- `request_disconnect`, 0x00
- `request_ping`, 0x01
- `request_echo`, 0x02

`request_echo` is assumed to be followed by some data that is to be sent back to the requesting client in a server `response_echo` packet.

#### Server -> Client

- `response_disconnect`, 0x00
- `response_ping`, 0x01
- `response_echo`, 0x02
- `order_disconnect`, 0x03

`response_ping` will carry a `VarUInt` along with it, denoting the time the original request was sent to the server. The total roundtrip can then be calculated clientside by `now() - packet_timestamp - ping_data`

#### Client -> Client

- `communication_text`, 0x00
- `communication_file`, 0x01

## Examples

### Client -> Client Text

```
[<packet_len as VarUInt>, 12, TestClient_1, 12, TestClient_2, <timestamp as VarUInt>, 0x00, "Test message from TestClient_1 to TestClient_2" as bytes]
```

<!--
- `Communication`, 0x00
  - `Text`, 0x00
  - `File`, 0x01
- `Order`, 0x01
  - `Disconnect`, 0x00 -->

<!-- - `Request`, 0x00
  - `Disconnect`, 0x00
  - `Ping`, 0x01
  - `Echo`, 0x02
- `Communication`, 0x01
  - `Text`, 0x00
  - `File`, 0x01 -->

<!-- > `Echo` is formed as `0x02`, `<varint echo msg size>`, `<echo msg>` -->

<!-- Valid data payloads:

- `Communication::{Text, File, ...}`
- `Request::{Ping, Echo, ...}`
- `Response::{...}` -->
