#!/usr/bin/env node
/**
 * make-license.mjs — offline machine-bound license generator for SAIKO SOLVER.
 *
 * Usage:
 *   node scripts/license/make-license.mjs init [bits]
 *       Generate a fresh RSA keypair.
 *         - PRIVATE key   -> scripts/license/private.pem   (SECRET, gitignored)
 *         - PUBLIC key    -> src-tauri/src/license_pub_key.pem (embedded in the exe)
 *
 *   node scripts/license/make-license.mjs code <MACHINE_ID> <days> [plan]
 *       Mint one license code for a machine ID. days = 0 means lifetime.
 *       Prints the code and copies it to the clipboard.
 *
 *   node scripts/license/make-license.mjs verify <CODE> <MACHINE_ID>
 *       Sanity-check a code against both keys (uses the embedded public key).
 *
 * The signed message is:  v1|<machine>|<expiryEpochSecs>|<plan>
 *   - machine:  24 uppercase hex chars (see license.rs machine_id())
 *   - expiry:   unix epoch seconds, 0 = never expires
 *   - plan:     free-form plan label (leave empty for none)
 * Code format:  <base64url(payload JSON)>.<base64url(signature)>
 * Signature:    RSA PKCS#1 v1.5 with SHA-256 (matches rsa crate verification).
 */

import { createPrivateKey, createPublicKey, generateKeyPairSync, sign, verify, randomUUID } from 'node:crypto'
import { mkdirSync, readFileSync, writeFileSync, existsSync } from 'node:fs'
import { resolve, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'
import { execSync } from 'node:child_process'

const __dirname = dirname(fileURLToPath(import.meta.url))
const ROOT = resolve(__dirname, '..', '..')
const PRIVATE_PEM = resolve(__dirname, 'private.pem')
const PUBLIC_PEM = resolve(__dirname, 'public.pem')
const EMBEDDED_PUBLIC_PEM = resolve(ROOT, 'src-tauri', 'src', 'license_pub_key.pem')
const PROTOCOL = 'v1'

function b64url(buf) {
  return Buffer.from(buf).toString('base64url')
}

function nowEpoch() {
  return Math.floor(Date.now() / 1000)
}

function canonicalMessage(machine, expiry, plan) {
  return `${PROTOCOL}|${machine.toLowerCase()}|${expiry}|${plan}`
}

function requirePrivateKey() {
  if (!existsSync(PRIVATE_PEM)) {
    console.error('Missing private key. Run: node scripts/license/make-license.mjs init')
    process.exit(1)
  }
  return createPrivateKey({ key: readFileSync(PRIVATE_PEM, 'utf8'), format: 'pem', type: 'pkcs1' })
}

function readPublicPem() {
  const file = existsSync(PUBLIC_PEM) ? PUBLIC_PEM : EMBEDDED_PUBLIC_PEM
  return readFileSync(file, 'utf8')
}

function copyToClipboard(text) {
  try {
    execSync(`powershell -NoProfile -c "Set-Clipboard -Value '${text.replace(/'/g, "''")}'"`, { stdio: 'ignore' })
    return true
  } catch {
    return false
  }
}

function cmdInit(bits) {
  const keySize = Number(bits ?? 2048)
  const { privateKey, publicKey } = generateKeyPairSync('rsa', {
    modulusLength: keySize,
    publicExponent: 0x10001,
  })

  mkdirSync(dirname(PRIVATE_PEM), { recursive: true })
  writeFileSync(PRIVATE_PEM, privateKey.export({ type: 'pkcs1', format: 'pem' }), 'utf8')
  writeFileSync(PUBLIC_PEM, publicKey.export({ type: 'pkcs1', format: 'pem' }), 'utf8')
  writeFileSync(EMBEDDED_PUBLIC_PEM, publicKey.export({ type: 'pkcs1', format: 'pem' }), 'utf8')

  console.log(`RSA-${keySize} keypair generated.`)
  console.log(`  PRIVATE: ${PRIVATE_PEM}   (KEEP SECRET — do not share)`)
  console.log(`  PUBLIC:  ${EMBEDDED_PUBLIC_PEM}   (embedded in the exe)`)
  console.log('The private key is gitignored. The public key file is committed.')
}

function cmdCode(machineId, days, plan) {
  const privateKey = requirePrivateKey()
  const machine = String(machineId ?? '').trim()
  if (!/^[0-9a-fA-F]{24}$/.test(machine)) {
    console.error('Machine ID must be exactly 24 hex characters (lowercase or uppercase).')
    process.exit(1)
  }
  const rawDays = Number(days ?? 0)
  const expires = rawDays > 0 ? nowEpoch() + rawDays * 86400 : 0
  const label = String(plan ?? '').trim()

  const payload = { v: 1, m: machine.toLowerCase(), e: expires, p: label }
  const payloadB64 = b64url(JSON.stringify(payload))
  const message = canonicalMessage(machine, expires, label)
  const signatureB64 = b64url(sign('RSA-SHA256', Buffer.from(message, 'utf8'), privateKey))

  const code = `${payloadB64}.${signatureB64}`

  console.log('License generated for machine: ' + machine)
  console.log('Expiry: ' + (expires === 0 ? 'lifetime' : `${rawDays} days (${new Date(expires * 1000).toISOString()})`))
  console.log('Plan:   ' + (label || '(none)'))
  console.log('')
  console.log('CODE:')
  console.log(code)
  console.log('')
  console.log(`Copied to clipboard: ${copyToClipboard(code) ? 'yes' : 'no'}`)
}

function cmdVerify(code, machineId) {
  const machine = String(machineId ?? '').trim().toLowerCase()
  const parts = String(code ?? '').trim().split('.')
  if (parts.length !== 2) {
    console.error('BAD FORMAT: code must be <payload>.<signature>')
    process.exit(1)
  }
  const [payloadB64, signatureB64] = parts
  const publicKey = createPublicKey({ key: readPublicPem(), format: 'pem', type: 'pkcs1' })

  let payload
  try {
    payload = JSON.parse(Buffer.from(payloadB64, 'base64url').toString('utf8'))
  } catch {
    console.error('BAD PAYLOAD: could not decode')
    process.exit(1)
  }
  if (payload.v !== 1 || typeof payload.m !== 'string' || typeof payload.e !== 'number') {
    console.error('BAD PAYLOAD: unsupported shape')
    process.exit(1)
  }

  const message = canonicalMessage(payload.m, payload.e, String(payload.p ?? ''))
  const ok = verify('RSA-SHA256', Buffer.from(message, 'utf8'), publicKey, Buffer.from(signatureB64, 'base64url'))
  if (!ok) {
    console.error('BAD SIGNATURE: public key rejected this code')
    process.exit(1)
  }

  const expiry = payload.e
  const expired = expiry !== 0 && expiry < nowEpoch()
  const machineOk = machine === '' || machine === payload.m

  console.log('Signature:      VALID')
  console.log('Machine (code): ' + payload.m + (machineOk ? '  (matches given machine)' : '  (MISMATCH with given machine!)'))
  console.log('Expiry:         ' + (expiry === 0 ? 'lifetime' : new Date(expiry * 1000).toISOString() + (expired ? '  (EXPIRED)' : '')))
  console.log('Plan:           ' + String(payload.p ?? '') || '(none)')

  if (expired || (machine !== '' && !machineOk)) {
    process.exit(1)
  }
}

const [command, ...args] = process.argv.slice(2)
switch (command) {
  case 'init':
    cmdInit(args[0])
    break
  case 'code':
    cmdCode(args[0], args[1], args[2])
    break
  case 'verify':
    cmdVerify(args[0], args[1])
    break
  default:
    console.log('Usage: node scripts/license/make-license.mjs <init|code|verify> [...]')
    break
}