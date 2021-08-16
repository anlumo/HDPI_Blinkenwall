const ws = require("nodejs-websocket")

let conn = ws.connect("ws://localhost:1337/blinkenwall", () => {
	conn.sendText(JSON.stringify({
		req: "1",
		cmd: "tox start",
	}), () => {
		process.exit(0)
	})
})

