// Web Worker that runs the woff2 WebAssembly converter entirely in the browser.
// The woff2.js + woff2.wasm are built from the latest google/woff2 using emscripten.
// No font data ever leaves the browser.

// Globals referenced by the emscripten EM_ASM blocks in api.cpp.
var _ptr = 0
var _length = 0

// Track the current conversion so progress messages can be routed to the
// correct request. The C-side api.cpp already throttles _progress() so it
// only calls EM_ASM when the integer percentage changes — at most ~100 calls
// per conversion.
var _currentId = 0

function _progress(percentage) {
  self.postMessage({ id: _currentId, type: 'progress', value: percentage })
}

// Configure the emscripten Module before importing the generated glue code.
// Suppress the library's internal stdout/stderr logging (e.g. "Compressed X to Y.").
var Module = {
  print: function () {},
  printErr: function () {},
  onRuntimeInitialized: function () {
    self.postMessage({ type: 'ready' })
    flushQueue()
  }
}

self.importScripts('woff2.js')

var ready = false
var queue = []

self.onmessage = function (event) {
  var msg = event.data
  if (msg.type !== 'convert') return

  if (!ready) {
    queue.push(msg)
    return
  }
  handleConvert(msg)
}

function flushQueue() {
  ready = true
  while (queue.length) {
    handleConvert(queue.shift())
  }
}

function handleConvert(msg) {
  var id = msg.id
  var to = msg.to
  _currentId = id
  try {
    var input = new Uint8Array(msg.buffer)
    var addr = Module._malloc(input.byteLength)
    if (!addr) {
      self.postMessage({ id: id, type: 'error', error: 'Out of memory' })
      return
    }
    Module.HEAP8.set(input, addr)
    var cFunc = to === 'ttf' ? Module._woff2_to_ttf : Module._ttf_to_woff2
    var outLen = cFunc(addr, input.byteLength)
    Module._free(addr)
    if (!outLen || !_ptr) {
      self.postMessage({ id: id, type: 'error', error: 'Conversion failed. The file may be invalid or corrupted.' })
      return
    }
    // Copy the result out of WASM heap into a standalone ArrayBuffer.
    // .slice() is required because Module.HEAPU8.buffer is the entire WASM
    // linear memory (potentially 16MB+); we only want outLen bytes.
    var output = new Uint8Array(Module.HEAPU8.buffer, _ptr, outLen).slice()
    Module._free(_ptr)
    _ptr = 0
    _length = 0
    // Send 100% progress right before the result.
    self.postMessage({ id: id, type: 'progress', value: 100 })
    self.postMessage({ id: id, type: 'done', buffer: output.buffer }, [output.buffer])
  } catch (err) {
    self.postMessage({ id: id, type: 'error', error: String(err && err.message ? err.message : err) })
  } finally {
    _currentId = 0
  }
}
