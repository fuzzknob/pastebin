import express from 'express'
import path from 'path'
import dayjs from 'dayjs'
import http from 'http'
import 'dotenv/config'
import { Server } from 'socket.io'

const PORT = 8000
const EXPIRE_AFTER_SECONDS = Number(process.env.EXPIRE_AFTER_SECONDS) || 900

const expressServer = express()
expressServer.set('view engine', 'ejs')
expressServer.set('views', path.join(__dirname, 'views'))


let data: {
  content: string
  expiryDate: dayjs.Dayjs
} | null = null

expressServer.get('/', (req, res) => {
  res.render('index', { content: data ? data.content : '', })
})

function wait(time: number) {
  return new Promise((res) => {
    setTimeout(() => res(true), time)
  })
}

async function watch() {
  while (true) {
    await wait(1000)
    if (!data) continue
    if (dayjs().isAfter(data.expiryDate)) {
      data = null
    }
  }
}

const httpServer = http.createServer(expressServer)
const io = new Server(httpServer)

io.on('connection', (socket) => {
  socket.on('paste', (content: string) => {
    if (!content) return data = null
    data = {
      content,
      expiryDate: dayjs().add(EXPIRE_AFTER_SECONDS, 'minutes')
    }
  })
})

watch()

httpServer.listen(PORT, () => {
  console.log(`Started server at http://localhost:${PORT}`)
})
