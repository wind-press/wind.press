// Native static-TTF-to-variable-font converter. The worker owns the Rust/WASM
// engine, so all font parsing and output construction stay on the user's device.

export interface VariableAxisInput {
  tag: string
  name: string
  minimum: number
  default: number
  maximum: number
}

export interface StaticFontMasterInput {
  name: string
  file: File
  location: number[]
}

export interface GlyphRepairInput {
  glyph: string
  strategy: 'freeze' | 'open_bar'
  letter?: string
  anchor?: 'left' | 'right'
  nubOverlap?: number
  minProtrude?: number
}

export interface StaticFontAnalysis {
  family: string | null
  weight: number | null
  italic: boolean
}

export interface VariableFontValidation {
  defaultMaster: number
  glyphCount: number
  unitsPerEm: number
  axisCount: number
  masterCount: number
  normalizedGlyphCount: number
  frozenGlyphCount: number
}

interface PendingRequest {
  resolve: (value: any) => void
  reject: (reason: Error) => void
}

let nextId = 0

export function useStaticToVariableConverter() {
  let worker: Worker | undefined
  let ready: Promise<void> | undefined
  const pending = new Map<number, PendingRequest>()

  function rejectPending(error: Error) {
    for (const request of pending.values()) request.reject(error)
    pending.clear()
  }

  function ensureWorker(): Promise<void> {
    if (ready) return ready
    if (!import.meta.client) {
      return Promise.reject(new Error('The variable font engine is available only in the browser.'))
    }

    ready = new Promise<void>((resolve, reject) => {
      worker = new Worker('/wasm/static-to-variable/static-to-variable-worker.js', { type: 'module' })
      worker.onmessage = (event) => {
        const message = event.data
        if (!message) return
        if (message.type === 'ready') {
          resolve()
          return
        }
        if (message.type === 'init-error') {
          const error = new Error(message.error || 'Could not initialize the variable font engine.')
          reject(error)
          rejectPending(error)
          return
        }
        const request = pending.get(message.id)
        if (!request) return
        pending.delete(message.id)
        if (message.type === 'error') {
          request.reject(new Error(message.error || 'Variable font conversion failed.'))
        } else {
          request.resolve(
            message.type === 'built'
              ? message.buffer
              : message.type === 'analyzed'
                ? message.reports
                : message.report
          )
        }
      }
      worker.onerror = (event) => {
        const error = new Error(event.message || 'The variable font worker stopped unexpectedly.')
        reject(error)
        rejectPending(error)
      }
    })
    return ready
  }

  async function run<T>(
    type: 'validate' | 'build',
    axes: VariableAxisInput[],
    masters: StaticFontMasterInput[],
    repairs: GlyphRepairInput[] = []
  ): Promise<T> {
    await ensureWorker()
    // Vue wraps values passed from a component in reactive Proxies. Web
    // Workers only accept structured-cloneable data, so take a plain snapshot
    // of the design space before reading and transferring the font files.
    const requestAxes = axes.map(axis => ({
      tag: axis.tag,
      name: axis.name,
      minimum: Number(axis.minimum),
      default: Number(axis.default),
      maximum: Number(axis.maximum)
    }))
    const requestMasters = masters.map(master => ({
      name: master.name,
      location: Array.from(master.location, coordinate => Number(coordinate)),
      file: master.file
    }))
    const requestRepairs = repairs.map(repair => ({
      glyph: String(repair.glyph).trim(),
      strategy: repair.strategy,
      letter: repair.letter?.trim() || undefined,
      anchor: repair.anchor,
      nubOverlap: repair.nubOverlap === undefined ? undefined : Number(repair.nubOverlap),
      minProtrude: repair.minProtrude === undefined ? undefined : Number(repair.minProtrude)
    }))
    const payload = await Promise.all(requestMasters.map(async master => ({
      name: master.name,
      location: master.location,
      buffer: await master.file.arrayBuffer()
    })))
    const id = ++nextId
    return new Promise<T>((resolve, reject) => {
      pending.set(id, { resolve, reject })
      worker!.postMessage({ type, id, axes: requestAxes, masters: payload, repairs: requestRepairs }, payload.map(master => master.buffer))
    })
  }

  function validate(axes: VariableAxisInput[], masters: StaticFontMasterInput[], repairs?: GlyphRepairInput[]) {
    return run<VariableFontValidation>('validate', axes, masters, repairs)
  }

  function build(axes: VariableAxisInput[], masters: StaticFontMasterInput[], repairs?: GlyphRepairInput[]) {
    return run<ArrayBuffer>('build', axes, masters, repairs)
  }

  async function analyze(files: File[]): Promise<StaticFontAnalysis[]> {
    await ensureWorker()
    const fonts = await Promise.all(files.map(async file => ({
      buffer: await file.arrayBuffer()
    })))
    const id = ++nextId
    return new Promise<StaticFontAnalysis[]>((resolve, reject) => {
      pending.set(id, { resolve, reject })
      worker!.postMessage({ type: 'analyze', id, fonts }, fonts.map(font => font.buffer))
    })
  }

  function dispose() {
    rejectPending(new Error('The variable font converter was closed.'))
    worker?.terminate()
    worker = undefined
    ready = undefined
  }

  return { analyze, validate, build, dispose }
}
