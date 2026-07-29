let enginePromise

async function loadEngine() {
  if (!enginePromise) {
    enginePromise = import('/wasm/static-to-variable/static-to-variable.js')
      .then(async (module) => {
        await module.default({ module_or_path: '/wasm/static-to-variable/static-to-variable_bg.wasm' })
        return module
      })
  }
  return enginePromise
}

function asRequest(message) {
  return {
    axes: message.axes,
    repairs: message.repairs || [],
    masters: message.masters.map((master) => ({
      name: master.name,
      location: master.location,
      bytes: new Uint8Array(master.buffer)
    }))
  }
}

self.onmessage = async (event) => {
  const message = event.data
  if (!message || !['analyze', 'validate', 'build'].includes(message.type)) return

  try {
    const engine = await loadEngine()
    if (message.type === 'analyze') {
      const reports = message.fonts.map((font) => engine.analyzeStaticFont(new Uint8Array(font.buffer)))
      self.postMessage({ type: 'analyzed', id: message.id, reports })
      return
    }

    const request = asRequest(message)
    if (message.type === 'validate') {
      const report = engine.validateVariableFontRequest(request)
      self.postMessage({ type: 'validated', id: message.id, report })
      return
    }

    const font = engine.buildVariableFont(request)
    // slice() detaches the small output buffer, never the WASM linear-memory
    // view returned by the binding.
    const buffer = font.slice().buffer
    self.postMessage({ type: 'built', id: message.id, buffer }, [buffer])
  } catch (error) {
    self.postMessage({
      type: 'error',
      id: message.id,
      error: error instanceof Error ? error.message : String(error)
    })
  }
}

loadEngine()
  .then(() => self.postMessage({ type: 'ready' }))
  .catch((error) => self.postMessage({
    type: 'init-error',
    error: error instanceof Error ? error.message : String(error)
  }))
