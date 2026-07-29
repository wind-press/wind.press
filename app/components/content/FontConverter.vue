<script setup lang="ts">
import { useWoff2Converter } from '~/composables/useWoff2Converter'
import prettyMs from 'pretty-ms'
import prettyBytes from 'pretty-bytes'

type Status = 'pending' | 'queued' | 'converting' | 'done' | 'error'

interface QueueItem {
  id: number
  file: File
  name: string
  size: number
  direction: 'woff2' | 'ttf'
  status: Status
  progress: number
  result: ArrayBuffer | null
  resultName: string
  error: string
  duration: number | null
}

const { convert, convertName } = useWoff2Converter()

const items = ref<QueueItem[]>([])
const dragOver = ref(false)
const inputRef = ref<HTMLInputElement | null>(null)
let seq = 0

const hasItems = computed(() => items.value.length > 0)
const isBusy = computed(() => items.value.some(i => i.status === 'converting' || i.status === 'queued'))
const hasResults = computed(() => items.value.some(i => i.status === 'done'))

const processedCount = computed(() => items.value.filter(i => i.status === 'done' || i.status === 'error').length)
const totalCount = computed(() => items.value.length)
const statsLabel = computed(() => `${processedCount.value}/${totalCount.value} processed`)

function detectDirection(name: string): 'woff2' | 'ttf' {
  return name.toLowerCase().endsWith('.woff2') ? 'ttf' : 'woff2'
}

function addFiles(fileList: FileList | File[]) {
  const accepted = Array.from(fileList).filter((f) => {
    const n = f.name.toLowerCase()
    return n.endsWith('.ttf') || n.endsWith('.woff2')
  })
  for (const file of accepted) {
    const direction = detectDirection(file.name)
    items.value.push({
      id: ++seq,
      file,
      name: file.name,
      size: file.size,
      direction,
      status: 'pending',
      progress: 0,
      result: null,
      resultName: convertName(file.name, direction),
      error: '',
      duration: null
    })
  }
}

function onDrop(event: DragEvent) {
  dragOver.value = false
  if (event.dataTransfer?.files) addFiles(event.dataTransfer.files)
}

function onInputChange(event: Event) {
  const target = event.target as HTMLInputElement
  if (target.files) addFiles(target.files)
  target.value = ''
}

async function convertItem(item: QueueItem) {
  // Mark as queued so the UI shows it's waiting for a worker.
  item.status = 'queued'
  item.progress = 0
  item.error = ''
  item.duration = null

  let startTime = 0
  try {
    const buffer = await item.file.arrayBuffer()

    // onStart fires when a worker actually picks up the job, not when it's
    // queued. The timer starts here so it only measures real conversion time.
    const result = await convert(
      buffer,
      item.direction,
      (p) => { item.progress = p },
      () => {
        item.status = 'converting'
        startTime = performance.now()
      }
    )
    item.result = result
    item.status = 'done'
    item.progress = 100
    item.duration = Math.round(performance.now() - startTime)
  } catch (err: any) {
    item.status = 'error'
    item.error = err?.message || 'Conversion failed.'
    item.duration = startTime > 0 ? Math.round(performance.now() - startTime) : null
  }
}

async function convertAll() {
  const pending = items.value.filter(i => i.status === 'pending' || i.status === 'error')
  // Run all conversions in parallel. The worker pool distributes them
  // across multiple WASM instances (one per CPU core, up to MAX_WORKERS).
  // Items beyond the pool size will show "Queued" until a worker frees up.
  await Promise.all(pending.map(item => convertItem(item)))
}

function download(item: QueueItem) {
  if (!item.result) return
  const blob = new Blob([item.result], { type: 'application/octet-stream' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = item.resultName
  document.body.appendChild(a)
  a.click()
  document.body.removeChild(a)
  URL.revokeObjectURL(url)
}

function downloadAll() {
  items.value.filter(i => i.status === 'done').forEach(download)
}

function removeItem(item: QueueItem) {
  items.value = items.value.filter(i => i.id !== item.id)
}

function clearAll() {
  items.value = []
}

function formatBytes(bytes: number): string {
  return prettyBytes(bytes)
}

function directionLabel(item: QueueItem): string {
  return item.direction === 'woff2' ? 'TTF → WOFF2' : 'WOFF2 → TTF'
}

function formatDuration(ms: number | null): string {
  if (ms === null) return ''
  return prettyMs(ms)
}

function sizeChangePct(item: QueueItem): number {
  if (!item.result || item.size === 0) return 0
  return Math.round((item.result.byteLength / item.size - 1) * 100)
}

function sizeChangeLabel(item: QueueItem): string {
  const pct = sizeChangePct(item)
  if (pct === 0) return '(±0%)'
  const sign = pct > 0 ? '+' : ''
  return `(${sign}${pct}%)`
}

function sizeChangeClass(item: QueueItem): string {
  const pct = sizeChangePct(item)
  if (pct < 0) return 'text-success'
  if (pct > 0) return 'text-warning'
  return 'text-muted'
}
</script>

<template>
  <div class="font-converter not-prose my-8">
    <div
      class="dropzone"
      :class="{ 'dropzone--active': dragOver }"
      role="button"
      tabindex="0"
      @click="inputRef?.click()"
      @keydown.enter.prevent="inputRef?.click()"
      @keydown.space.prevent="inputRef?.click()"
      @dragover.prevent="dragOver = true"
      @dragleave.prevent="dragOver = false"
      @drop.prevent="onDrop"
    >
      <input
        ref="inputRef"
        type="file"
        accept=".ttf,.woff2"
        multiple
        class="hidden"
        @change="onInputChange"
      >
      <UIcon name="i-lucide-upload-cloud" class="dropzone__icon" />
      <p class="dropzone__title">
        Add <code>.ttf</code> or <code>.woff2</code> files
      </p>
      <p class="dropzone__subtitle">
        TTF becomes WOFF2 and WOFF2 becomes TTF. Files stay on your device.
      </p>
    </div>

    <div v-if="hasItems" class="mt-6 space-y-4">
      <div class="flex flex-wrap items-center gap-3">
        <UButton
          icon="i-lucide-zap"
          color="primary"
          :loading="isBusy"
          :disabled="isBusy"
          @click="convertAll"
        >
          Convert all
        </UButton>
        <UButton
          v-if="hasResults"
          icon="i-lucide-download"
          variant="subtle"
          color="neutral"
          @click="downloadAll"
        >
          Download all
        </UButton>
        <UButton
          icon="i-lucide-trash-2"
          variant="ghost"
          color="neutral"
          @click="clearAll"
        >
          Clear
        </UButton>

        <div v-if="isBusy || processedCount > 0" class="queue__stats">
          <UIcon name="i-lucide-list-checks" class="queue__stats-icon" />
          <span>{{ statsLabel }}</span>
          <span v-if="isBusy" class="queue__stats-spinner">
            <UIcon name="i-lucide-loader-circle" class="animate-spin" />
          </span>
        </div>
      </div>

      <ul class="queue">
        <li v-for="item in items" :key="item.id" class="queue__item">
          <div class="queue__main">
            <div class="queue__name-row">
              <UIcon
                :name="item.direction === 'woff2' ? 'i-lucide-file-type' : 'i-lucide-file-type-2'"
                class="queue__file-icon"
              />
              <div class="min-w-0">
                <div class="queue__name">
                  {{ item.name }}
                </div>
                <div class="queue__meta">
                  <UBadge :label="directionLabel(item)" size="sm" variant="subtle" color="primary" />
                  <span>{{ formatBytes(item.size) }}</span>
                  <span v-if="item.status === 'done' && item.result" class="queue__meta-stats">
                    → {{ formatBytes(item.result.byteLength) }}
                    <span :class="sizeChangeClass(item)">
                      {{ sizeChangeLabel(item) }}
                    </span>
                    <UIcon name="i-lucide-timer" class="queue__meta-icon" />
                    {{ formatDuration(item.duration) }}
                  </span>
                </div>
              </div>
            </div>

            <div class="queue__actions">
              <UButton
                v-if="item.status === 'pending' || item.status === 'error'"
                size="sm"
                variant="soft"
                color="primary"
                icon="i-lucide-zap"
                @click="convertItem(item)"
              >
                Convert
              </UButton>
              <UButton
                v-if="item.status === 'done'"
                size="sm"
                variant="soft"
                color="success"
                icon="i-lucide-download"
                @click="download(item)"
              >
                Download
              </UButton>
              <UButton
                size="sm"
                variant="ghost"
                color="neutral"
                icon="i-lucide-x"
                @click="removeItem(item)"
              />
            </div>
          </div>

          <div v-if="item.status === 'queued'" class="queue__queued-hint">
            <UIcon name="i-lucide-hourglass" class="queue__hint-icon" />
            Waiting for a free worker...
          </div>

          <div v-if="item.status === 'converting'" class="queue__progress">
            <UProgress :model-value="item.progress" size="sm" />
          </div>

          <div v-if="item.status === 'error'" class="queue__error">
            <UIcon name="i-lucide-triangle-alert" /> {{ item.error }}
          </div>

          <div v-if="item.status === 'done'" class="queue__success">
            <UIcon name="i-lucide-circle-check" /> Converted to <code>{{ item.resultName }}</code>
          </div>
        </li>
      </ul>
    </div>
  </div>
</template>

<style scoped>
.font-converter {
  font-family: inherit;
}

.dropzone {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
  gap: 0.5rem;
  padding: 2.5rem 1.5rem;
  border: 2px dashed var(--ui-border);
  border-radius: calc(var(--ui-radius) * 2);
  background-color: var(--ui-bg-muted);
  cursor: pointer;
  transition: border-color 0.15s ease, background-color 0.15s ease;
}

.dropzone:hover,
.dropzone--active {
  border-color: var(--ui-primary);
  background-color: color-mix(in oklab, var(--ui-primary) 8%, var(--ui-bg-muted));
}

.dropzone__icon {
  width: 2.5rem;
  height: 2.5rem;
  color: var(--ui-text-muted);
}

.dropzone__title {
  font-weight: 600;
  font-size: 1rem;
  margin: 0;
}

.dropzone__subtitle {
  font-size: 0.875rem;
  color: var(--ui-text-muted);
  margin: 0;
}

.dropzone code {
  font-weight: 600;
}

.queue {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.queue__item {
  border: 1px solid var(--ui-border);
  border-radius: var(--ui-radius);
  padding: 0.875rem 1rem;
  background-color: var(--ui-bg);
}

.queue__main {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
}

.queue__name-row {
  display: flex;
  align-items: flex-start;
  gap: 0.625rem;
  min-width: 0;
}

.queue__file-icon {
  width: 1.25rem;
  height: 1.25rem;
  color: var(--ui-primary);
  flex-shrink: 0;
  margin-top: 0.125rem;
}

.queue__name {
  font-weight: 600;
  font-size: 0.9rem;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.queue__meta {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.75rem;
  color: var(--ui-text-muted);
  margin-top: 0.25rem;
  flex-wrap: wrap;
}

.queue__meta-icon {
  width: 0.875rem;
  height: 0.875rem;
}

.queue__meta-stats {
  display: flex;
  align-items: center;
  gap: 0.375rem;
}

.queue__actions {
  display: flex;
  align-items: center;
  gap: 0.25rem;
  flex-shrink: 0;
}

.queue__stats {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  font-size: 0.8rem;
  color: var(--ui-text-muted);
  margin-left: auto;
}

.queue__stats-icon {
  width: 0.875rem;
  height: 0.875rem;
}

.queue__stats-spinner {
  display: inline-flex;
  align-items: center;
}

.queue__queued-hint {
  margin-top: 0.5rem;
  display: flex;
  align-items: center;
  gap: 0.375rem;
  font-size: 0.8rem;
  color: var(--ui-text-muted);
}

.queue__hint-icon {
  width: 0.875rem;
  height: 0.875rem;
}

.queue__progress {
  margin-top: 0.75rem;
}

.queue__error {
  margin-top: 0.5rem;
  display: flex;
  align-items: center;
  gap: 0.375rem;
  font-size: 0.8rem;
  color: var(--ui-error);
}

.queue__success {
  margin-top: 0.5rem;
  display: flex;
  align-items: center;
  gap: 0.375rem;
  font-size: 0.8rem;
  color: var(--ui-success);
}
</style>
