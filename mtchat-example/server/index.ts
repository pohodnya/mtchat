/**
 * Webhook receiver server for mtchat-example
 *
 * Receives webhooks from mtchat-rust and broadcasts to browsers via SSE
 */

import express from 'express'
import cors from 'cors'

const app = express()
const PORT = process.env.PORT || 3001

// Store SSE clients
const clients: Set<express.Response> = new Set()

// Middleware
app.use(cors())
app.use(express.json())

// SSE endpoint for browsers
app.get('/events', (req, res) => {
  // Set headers for SSE
  res.setHeader('Content-Type', 'text/event-stream')
  res.setHeader('Cache-Control', 'no-cache')
  res.setHeader('Connection', 'keep-alive')
  res.setHeader('Access-Control-Allow-Origin', '*')

  // Send initial connection message
  res.write('data: {"type":"connected"}\n\n')

  // Add to clients
  clients.add(res)
  console.log(`[SSE] Client connected. Total: ${clients.size}`)

  // Remove on disconnect
  req.on('close', () => {
    clients.delete(res)
    console.log(`[SSE] Client disconnected. Total: ${clients.size}`)
  })
})

// Webhook endpoint - receives from mtchat-rust
app.post('/webhook', (req, res) => {
  const event = req.body

  console.log(`[Webhook] Received: ${event.type}`)
  console.log(JSON.stringify(event, null, 2))

  // Broadcast to all SSE clients
  const data = JSON.stringify(event)
  clients.forEach(client => {
    client.write(`data: ${data}\n\n`)
  })

  // Always respond 200 to webhook
  res.status(200).json({ ok: true })
})

// Health check
app.get('/health', (_req, res) => {
  res.json({ status: 'ok', clients: clients.size })
})

app.listen(PORT, () => {
  console.log(`[Webhook Server] Running on http://localhost:${PORT}`)
  console.log(`[Webhook Server] SSE endpoint: http://localhost:${PORT}/events`)
  console.log(`[Webhook Server] Webhook endpoint: http://localhost:${PORT}/webhook`)
})
