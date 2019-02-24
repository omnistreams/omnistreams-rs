const WebSocket = require('ws')

const ws = new WebSocket('ws://127.0.0.1:9001')

ws.binaryType = 'arraybuffer'

ws.on('open', () => {
  ws.send(new Uint8Array([0, 0]))
  ws.send(new Uint8Array([1, 0, 65]))
  ws.send(new Uint8Array([1, 0, 66]))
  ws.send(new Uint8Array([1, 0, 67, 68]))
  ws.send(new Uint8Array([2, 0]))
})
