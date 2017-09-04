const ws = require("nodejs-websocket")
const getStdin = require('get-stdin');

getStdin().then(str => {
  let conn = ws.connect("ws://localhost:1337/blinkenwall", () => {
    conn.sendText(JSON.stringify({
      req: "1",
      cmd: "show poetry",
      text: str,
    }), () => {
      process.exit(0)
    })
  })
})
