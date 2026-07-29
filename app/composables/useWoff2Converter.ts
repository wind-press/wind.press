// Composable to convert TTF <-> WOFF2 entirely in the browser via a pool of
// Web Workers running the woff2 WebAssembly module. No font data is uploaded.
//
// Multiple workers allow parallel conversion when the user uploads many fonts.
// The pool size is capped at navigator.hardwareConcurrency (with a sane max)
// since each worker loads its own WASM instance (~700KB + runtime memory).
//
// A concurrency semaphore ensures that when more jobs are submitted than
// workers available, excess jobs truly wait (the `onStart` callback fires
// only when a worker picks up the job, not when it's queued).

export type ConvertDirection = 'woff2' | 'ttf'

interface PendingRequest {
  resolve: (buffer: ArrayBuffer) => void
  reject: (error: Error) => void
  onProgress?: (percentage: number) => void
}

interface WorkerSlot {
  worker: Worker
  ready: boolean
  readyPromise: Promise<void>
  busy: boolean
}

const MAX_WORKERS = 4

function getPoolSize(): number {
  if (typeof navigator === 'undefined') return 1
  const cores = navigator.hardwareConcurrency || 2
  return Math.min(Math.max(1, cores - 1), MAX_WORKERS)
}

export function useWoff2Converter() {
  let pool: WorkerSlot[] = []
  let poolSize = 1
  let seq = 0
  const pending = new Map<number, PendingRequest>()
  let initialized = false

  // Semaphore: track how many workers are free.
  let freeWorkers = 0

  // Queue of jobs waiting for a free worker.
  interface QueuedJob {
    buffer: ArrayBuffer
    to: ConvertDirection
    onProgress?: (percentage: number) => void
    onStart?: () => void
    resolve: (buffer: ArrayBuffer) => void
    reject: (error: Error) => void
  }
  const queue: QueuedJob[] = []

  function createWorker(index: number): WorkerSlot {
    const slot: Partial<WorkerSlot> = {}

    slot.readyPromise = new Promise<void>((resolve, reject) => {
      const worker = new Worker('/wasm/woff2-worker.js')

      slot.worker = worker
      slot.ready = false
      slot.busy = false

      worker.onerror = (event) => {
        reject(new Error(event.message || 'Failed to load the converter worker.'))
      }

      worker.onmessage = (event) => {
        const msg = event.data
        if (!msg) return

        if (msg.type === 'ready') {
          slot.ready = true
          resolve()
          return
        }

        if (msg.type === 'progress') {
          const req = pending.get(msg.id)
          if (req?.onProgress) req.onProgress(msg.value)
          return
        }

        if (msg.type === 'done' || msg.type === 'error') {
          const req = pending.get(msg.id)
          if (!req) return
          pending.delete(msg.id)

          // Free this worker and drain the queue.
          slot.busy = false
          freeWorkers++
          drainQueue()

          if (msg.type === 'done') {
            req.resolve(msg.buffer)
          } else {
            req.reject(new Error(msg.error || 'Conversion failed.'))
          }
        }
      }
    })

    return slot as WorkerSlot
  }

  function drainQueue() {
    while (queue.length > 0 && freeWorkers > 0) {
      const job = queue.shift()!
      dispatch(job)
    }
  }

  function dispatch(job: QueuedJob) {
    // Find a free worker.
    const workerIndex = pool.findIndex(s => !s.busy)
    if (workerIndex === -1) {
      // Should not happen if freeWorkers > 0, but guard anyway.
      queue.unshift(job)
      return
    }

    pool[workerIndex].busy = true
    freeWorkers--

    if (job.onStart) job.onStart()

    const id = ++seq
    pending.set(id, {
      resolve: job.resolve,
      reject: job.reject,
      onProgress: job.onProgress
    })

    pool[workerIndex].worker.postMessage(
      { type: 'convert', id, to: job.to, buffer: job.buffer },
      [job.buffer]
    )
  }

  async function ensurePool(): Promise<void> {
    if (initialized) {
      await Promise.all(pool.map(s => s.readyPromise))
      return
    }

    if (!import.meta.client) {
      throw new Error('Web Workers are only available in the browser.')
    }

    poolSize = getPoolSize()
    freeWorkers = poolSize
    pool = Array.from({ length: poolSize }, (_, i) => createWorker(i))
    initialized = true

    await Promise.all(pool.map(s => s.readyPromise))
  }

  async function convert(
    buffer: ArrayBuffer,
    to: ConvertDirection,
    onProgress?: (percentage: number) => void,
    onStart?: () => void
  ): Promise<ArrayBuffer> {
    await ensurePool()

    return new Promise<ArrayBuffer>((resolve, reject) => {
      const job: QueuedJob = { buffer, to, onProgress, onStart, resolve, reject }

      if (freeWorkers > 0) {
        dispatch(job)
      } else {
        // All workers busy — queue the job. It will be dispatched when a
        // worker finishes (drainQueue is called from the worker's onmessage).
        queue.push(job)
      }
    })
  }

  function convertName(name: string, extension: ConvertDirection): string {
    const i = name.lastIndexOf('.')
    return (i >= 0 ? name.slice(0, i) : name) + '.' + extension
  }

  return {
    convert,
    convertName,
    poolSize: () => poolSize,
    isReady: () => initialized && pool.every(s => s.ready)
  }
}
