<script setup lang="ts">
import prettyBytes from 'pretty-bytes'
import {
  useStaticToVariableConverter,
  type GlyphRepairInput,
  type StaticFontAnalysis,
  type StaticFontMasterInput,
  type VariableAxisInput,
  type VariableFontValidation
} from '~/composables/useStaticToVariableConverter'

type Status = 'idle' | 'validating' | 'building' | 'done' | 'error'

interface MasterRow extends StaticFontMasterInput {
  id: number
  analysis?: StaticFontAnalysis
}

const { analyze, validate, build: buildNative, dispose } = useStaticToVariableConverter()

const axes = ref<VariableAxisInput[]>([
  { tag: 'wght', name: 'Weight', minimum: 100, default: 400, maximum: 900 }
])
const masters = ref<MasterRow[]>([])
const repairs = ref<GlyphRepairInput[]>([])
const dragOver = ref(false)
const inputRef = ref<HTMLInputElement | null>(null)
const status = ref<Status>('idle')
const error = ref('')
const result = ref<ArrayBuffer | null>(null)
const validation = ref<VariableFontValidation | null>(null)
const outputName = ref('VariableFont.ttf')
const outputNameIsAutomatic = ref(true)
const isDetecting = ref(false)
const detectedDesignSpace = ref(false)
const automaticDesignSpace = ref(true)
const previewText = ref('Sphinx of black quartz, judge my vow. 0123456789')
const previewLocation = ref<number[]>([])
const previewReady = ref(false)
const previewError = ref('')
const previewFamily = `Yabe Variable Preview ${Math.random().toString(36).slice(2)}`
let previewFace: FontFace | undefined
let sequence = 0

const isBusy = computed(() => status.value === 'validating' || status.value === 'building')
const canBuild = computed(() => masters.value.length >= 2 && !isBusy.value)
const hasOutput = computed(() => status.value === 'done' && result.value)
const previewStyle = computed(() => ({
  fontFamily: `"${previewFamily}"`,
  fontVariationSettings: axes.value
    .map((axis, index) => `'${axis.tag}' ${previewLocation.value[index] ?? axis.default}`)
    .join(', ')
}))

function newLocation() {
  return axes.value.map(axis => axis.default)
}

function outputNameFor(family: string | null | undefined, fileName: string) {
  const fileBase = fileName.replace(/\.[^.]+$/, '')
  const suggested = family?.trim() || fileBase || 'VariableFont'
  const safeBase = suggested
    .normalize('NFKC')
    .replace(/[<>:"/\\|?*]/g, '')
    .split('')
    .filter(character => character.charCodeAt(0) >= 0x20)
    .join('')
    .replace(/\s+/g, ' ')
    .replace(/[. ]+$/, '')
  return `${safeBase || 'VariableFont'} Variable.ttf`
}

async function addFiles(files: FileList | File[]) {
  const ttfFiles = Array.from(files).filter(file => file.name.toLowerCase().endsWith('.ttf'))
  const isFirstSelection = masters.value.length === 0
  const added = ttfFiles.map(file => ({
    id: ++sequence,
    name: file.name.replace(/\.[^.]+$/, ''),
    file,
    location: newLocation()
  }))
  masters.value.push(...added)
  if (ttfFiles.length && isFirstSelection) {
    outputName.value = outputNameFor(null, ttfFiles[0].name)
    outputNameIsAutomatic.value = true
  }
  clearOutput()

  if (!ttfFiles.length) return
  isDetecting.value = true
  try {
    const analyses = await analyze(ttfFiles)
    for (const [index, analysis] of analyses.entries()) {
      const master = masters.value.find(item => item.id === added[index].id)
      if (master) master.analysis = analysis
    }
    if (isFirstSelection && outputNameIsAutomatic.value) {
      outputName.value = outputNameFor(analyses.find(analysis => analysis.family)?.family, ttfFiles[0].name)
    }
    applyDetectedDesignSpace()
  } catch {
    // Analysis is an assistive convenience. A font can still be configured
    // manually and receive the full validation before it is built.
  } finally {
    isDetecting.value = false
  }
}

function onDrop(event: DragEvent) {
  dragOver.value = false
  if (event.dataTransfer?.files) addFiles(event.dataTransfer.files)
}

function onInputChange(event: Event) {
  const input = event.target as HTMLInputElement
  if (input.files) addFiles(input.files)
  input.value = ''
}

function removeMaster(id: number) {
  masters.value = masters.value.filter(master => master.id !== id)
  applyDetectedDesignSpace()
  clearOutput()
}

function addRepair(strategy: GlyphRepairInput['strategy']) {
  repairs.value.push(strategy === 'open_bar'
    ? { glyph: '$', strategy, letter: 'S', anchor: 'left' }
    : { glyph: '', strategy })
  clearOutput()
}

function removeRepair(index: number) {
  repairs.value.splice(index, 1)
  clearOutput()
}

function addAxis() {
  markDesignSpaceEdited()
  const suggestions = [
    { tag: 'wdth', name: 'Width', minimum: 75, default: 100, maximum: 125 },
    { tag: 'opsz', name: 'Optical Size', minimum: 8, default: 14, maximum: 72 },
    { tag: 'GRAD', name: 'Grade', minimum: -100, default: 0, maximum: 100 },
    { tag: 'XTRA', name: 'Xtra', minimum: 0, default: 50, maximum: 100 }
  ]
  const axis = suggestions.find(item => !axes.value.some(existing => existing.tag === item.tag))
    || { tag: `X${String(axes.value.length).padStart(3, '0')}`, name: 'Custom axis', minimum: -1, default: 0, maximum: 1 }
  axes.value.push({ ...axis })
  masters.value.forEach(master => master.location.push(axis.default))
  clearOutput()
}

function removeAxis(index: number) {
  if (axes.value.length === 1) return
  markDesignSpaceEdited()
  axes.value.splice(index, 1)
  masters.value.forEach(master => master.location.splice(index, 1))
  clearOutput()
}

function markDesignSpaceEdited() {
  automaticDesignSpace.value = false
  detectedDesignSpace.value = false
  clearOutput()
}

function usableWeight(weight: number | null | undefined) {
  // OS/2 permits 1–1000, but values below 100 are not useful `wght` axis
  // coordinates. Some legacy Italic fonts contain a stray value such as 5;
  // using that value would put the italic master hundreds of weight units
  // away from its upright counterpart and distort intermediate instances.
  return Number.isFinite(weight) && weight! >= 100 && weight! <= 1000
}

function inferredItalicWeight() {
  const uprightWeights = masters.value
    .filter(master => !master.analysis?.italic && usableWeight(master.analysis?.weight))
    .map(master => master.analysis!.weight!)

  return [...uprightWeights].sort((left, right) => Math.abs(left - 400) - Math.abs(right - 400))[0] ?? 400
}

function automaticWeight(master: MasterRow, italicFallback = inferredItalicWeight()) {
  if (usableWeight(master.analysis?.weight)) return master.analysis!.weight!
  return master.analysis?.italic ? italicFallback : undefined
}

function applyDetectedDesignSpace() {
  if (!automaticDesignSpace.value || masters.value.length < 2 || masters.value.some(master => !master.analysis)) return

  const italicFallback = inferredItalicWeight()
  const weights = masters.value.map(master => automaticWeight(master, italicFallback))
  const allWeightsKnown = weights.every((weight): weight is number => weight !== undefined)
  const hasWeightVariation = allWeightsKnown
    && new Set(weights).size > 1
  const hasItalicVariation = new Set(masters.value.map(master => master.analysis!.italic)).size > 1
  if (!hasWeightVariation && !hasItalicVariation) return

  const defaultIndex = masters.value
    .map((master, index) => ({ master, index, weight: weights[index] }))
    .sort((left, right) => {
      if (hasItalicVariation && left.master.analysis!.italic !== right.master.analysis!.italic) {
        return Number(left.master.analysis!.italic) - Number(right.master.analysis!.italic)
      }
      const leftDistance = allWeightsKnown ? Math.abs(left.weight! - 400) : 0
      const rightDistance = allWeightsKnown ? Math.abs(right.weight! - 400) : 0
      return leftDistance - rightDistance
    })[0]
    .index

  const defaultMaster = masters.value[defaultIndex]
  const defaultWeight = weights[defaultIndex]
  const detectedAxes: VariableAxisInput[] = []
  if (hasWeightVariation) {
    detectedAxes.push({
      tag: 'wght',
      name: 'Weight',
      minimum: Math.min(...weights),
      default: defaultWeight!,
      maximum: Math.max(...weights)
    })
  }
  if (hasItalicVariation) {
    detectedAxes.push({
      tag: 'ital',
      name: 'Italic',
      minimum: 0,
      default: defaultMaster.analysis!.italic ? 1 : 0,
      maximum: 1
    })
  }

  axes.value = detectedAxes
  masters.value.forEach((master, index) => {
    master.location = detectedAxes.map((axis) => {
      if (axis.tag === 'wght') return weights[index]!
      return master.analysis!.italic ? 1 : 0
    })
  })
  detectedDesignSpace.value = true
  clearOutput()
}

function detectedMetadata(master: MasterRow) {
  if (!master.analysis) return ''
  const details = []
  if (master.analysis.weight !== null) {
    const fallback = inferredItalicWeight()
    const displayWeight = !usableWeight(master.analysis.weight) && master.analysis.italic
      ? `${master.analysis.weight} (using ${fallback})`
      : String(master.analysis.weight)
    details.push(displayWeight)
  }
  details.push(master.analysis.italic ? 'italic' : 'upright')
  return `Detected: ${details.join(' · ')}`
}

function clearOutput() {
  clearPreview()
  result.value = null
  validation.value = null
  if (status.value !== 'validating' && status.value !== 'building') status.value = 'idle'
  error.value = ''
}

function clearPreview() {
  if (previewFace && import.meta.client) document.fonts.delete(previewFace)
  previewFace = undefined
  previewReady.value = false
  previewError.value = ''
  previewLocation.value = []
}

async function loadPreview(fontData: ArrayBuffer) {
  clearPreview()
  if (!import.meta.client) return
  try {
    const font = new FontFace(previewFamily, fontData.slice(0))
    await font.load()
    document.fonts.add(font)
    previewFace = font
    previewLocation.value = axes.value.map(axis => axis.default)
    previewReady.value = true
  } catch {
    previewError.value = 'The font was built, but this browser could not load its preview.'
  }
}

async function buildFont() {
  if (!canBuild.value) return
  error.value = ''
  clearPreview()
  result.value = null
  validation.value = null
  try {
    status.value = 'validating'
    validation.value = await validate(axes.value, masters.value, repairs.value)
    status.value = 'building'
    result.value = await buildNative(axes.value, masters.value, repairs.value)
    await loadPreview(result.value)
    status.value = 'done'
  } catch (cause: any) {
    status.value = 'error'
    error.value = cause?.message || 'Could not create the variable font.'
  }
}

function download() {
  if (!result.value) return
  const blob = new Blob([result.value], { type: 'font/ttf' })
  const url = URL.createObjectURL(blob)
  const link = document.createElement('a')
  link.href = url
  link.download = outputName.value.trim() || 'VariableFont.ttf'
  document.body.append(link)
  link.click()
  link.remove()
  URL.revokeObjectURL(url)
}

function formatBytes(bytes: number) {
  return prettyBytes(bytes)
}

function previewStep(axis: VariableAxisInput) {
  const range = axis.maximum - axis.minimum
  return axis.tag === 'ital' || Number.isInteger(range) ? 1 : Math.abs(range) / 100
}

onScopeDispose(() => {
  clearPreview()
  dispose()
})
</script>

<template>
  <div class="variable-font-converter not-prose my-8">
    <div
      class="variable-dropzone"
      :class="{ 'variable-dropzone--active': dragOver }"
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
        accept=".ttf"
        multiple
        class="hidden"
        @change="onInputChange"
      >
      <UIcon name="i-lucide-files" class="variable-dropzone__icon" />
      <p class="variable-dropzone__title">
        Add two or more static <code>.ttf</code> masters
      </p>
      <p class="variable-dropzone__subtitle">
        Choose files from the same family. They stay on your device.
      </p>
    </div>

    <div v-if="masters.length" class="variable-workspace">
      <section class="variable-panel">
        <div class="variable-panel__heading">
          <div>
            <p class="variable-panel__eyebrow">
              Design space
            </p>
            <h3>Axes</h3>
          </div>
          <UButton size="sm" variant="soft" color="neutral" icon="i-lucide-plus" @click="addAxis">
            Add axis
          </UButton>
        </div>

        <div class="axis-list">
          <div v-for="(axis, index) in axes" :key="`${axis.tag}-${index}`" class="axis-row">
            <label><span>Tag</span><input v-model.trim="axis.tag" maxlength="4" @change="markDesignSpaceEdited"></label>
            <label><span>Name</span><input v-model.trim="axis.name" @change="markDesignSpaceEdited"></label>
            <label><span>Min</span><input v-model.number="axis.minimum" type="number" step="any" @change="markDesignSpaceEdited"></label>
            <label><span>Default</span><input v-model.number="axis.default" type="number" step="any" @change="markDesignSpaceEdited"></label>
            <label><span>Max</span><input v-model.number="axis.maximum" type="number" step="any" @change="markDesignSpaceEdited"></label>
            <UButton
              size="xs"
              variant="ghost"
              color="neutral"
              icon="i-lucide-trash-2"
              :disabled="axes.length === 1"
              :aria-label="`Remove ${axis.name || axis.tag} axis`"
              @click="removeAxis(index)"
            />
          </div>
        </div>
      </section>

      <section class="variable-panel">
        <div class="variable-panel__heading">
          <div>
            <p class="variable-panel__eyebrow">
              Source files
            </p>
            <h3>Masters</h3>
          </div>
          <span class="variable-panel__count">{{ masters.length }} master{{ masters.length === 1 ? '' : 's' }}</span>
        </div>

        <div class="master-list">
          <div v-for="master in masters" :key="master.id" class="master-row">
            <div class="master-row__file">
              <UIcon name="i-lucide-file-type-2" />
              <div>
                <strong>{{ master.file.name }}</strong>
                <span>{{ detectedMetadata(master) || formatBytes(master.file.size) }}</span>
              </div>
            </div>
            <label class="master-row__name"><span>Instance name</span><input v-model.trim="master.name" @change="clearOutput"></label>
            <div class="master-row__locations">
              <label v-for="(axis, index) in axes" :key="axis.tag">
                <span>{{ axis.tag }}</span>
                <input v-model.number="master.location[index]" type="number" step="any" @change="markDesignSpaceEdited">
              </label>
            </div>
            <UButton
              size="xs"
              variant="ghost"
              color="neutral"
              icon="i-lucide-x"
              :aria-label="`Remove ${master.file.name}`"
              @click="removeMaster(master.id)"
            />
          </div>
        </div>
      </section>

      <section class="variable-panel">
        <div class="variable-panel__heading">
          <div>
            <p class="variable-panel__eyebrow">
              Optional repairs
            </p>
            <h3>Glyph rules</h3>
          </div>
          <div class="variable-panel__buttons">
            <UButton size="sm" variant="soft" color="neutral" icon="i-lucide-plus" @click="addRepair('open_bar')">
              Open bar
            </UButton>
            <UButton size="sm" variant="soft" color="neutral" icon="i-lucide-plus" @click="addRepair('freeze')">
              Freeze glyph
            </UButton>
          </div>
        </div>
        <p class="variable-detection">
          Use a production glyph name, a Unicode selector such as <code>U+0024</code>, or one character such as <code>$</code>. Open bar rebuilds an intentional <code>$</code> or <code>¢</code>-style design from a bare letter and two bar stubs; freeze keeps one glyph at the default outline.
        </p>
        <div v-if="repairs.length" class="repair-list">
          <div v-for="(repair, index) in repairs" :key="`${repair.strategy}-${index}`" class="repair-row">
            <label><span>Glyph</span><input v-model.trim="repair.glyph" placeholder="$ or U+0024" @change="clearOutput"></label>
            <label><span>Rule</span>
              <select v-model="repair.strategy" @change="clearOutput">
                <option value="open_bar">Open bar</option>
                <option value="freeze">Freeze at default</option>
              </select>
            </label>
            <template v-if="repair.strategy === 'open_bar'">
              <label><span>Bare letter</span><input v-model.trim="repair.letter" placeholder="S" @change="clearOutput"></label>
              <label><span>Anchor</span>
                <select v-model="repair.anchor" @change="clearOutput">
                  <option value="left">Left</option>
                  <option value="right">Right</option>
                </select>
              </label>
              <label><span>Nub overlap</span>
                <input
                  v-model.number="repair.nubOverlap"
                  type="number"
                  min="0"
                  step="any"
                  placeholder="30"
                  @change="clearOutput"
                >
              </label>
              <label><span>Min protrusion</span>
                <input
                  v-model.number="repair.minProtrude"
                  type="number"
                  min="0"
                  step="any"
                  placeholder="70"
                  @change="clearOutput"
                >
              </label>
            </template>
            <UButton
              size="xs"
              variant="ghost"
              color="neutral"
              icon="i-lucide-trash-2"
              :aria-label="`Remove ${repair.glyph || 'glyph'} repair`"
              @click="removeRepair(index)"
            />
          </div>
        </div>
      </section>

      <p v-if="isDetecting || detectedDesignSpace" class="variable-detection" aria-live="polite">
        {{ isDetecting ? 'Reading font details…' : 'Weight and italic values are ready to review.' }}
      </p>

      <section class="variable-actions">
        <div>
          <label class="variable-output-name"><span>Output file name</span><input v-model.trim="outputName" @input="outputNameIsAutomatic = false"></label>
          <p>Suggested from the first font. You can rename it.</p>
        </div>
        <div class="variable-actions__buttons">
          <UButton size="lg" icon="i-lucide-sparkles" :loading="isBusy" :disabled="!canBuild" @click="buildFont">
            {{ status === 'validating' ? 'Checking masters…' : status === 'building' ? 'Creating variable font…' : 'Create variable font' }}
          </UButton>
          <UButton
            v-if="hasOutput"
            size="lg"
            color="success"
            variant="soft"
            icon="i-lucide-download"
            @click="download"
          >
            Download TTF
          </UButton>
        </div>
      </section>

      <div v-if="validation && status === 'done'" class="variable-result variable-result--success">
        <UIcon name="i-lucide-circle-check" />
        <span>
          Created {{ validation.glyphCount.toLocaleString() }} glyphs from {{ validation.masterCount }} masters across {{ validation.axisCount }} axis{{ validation.axisCount === 1 ? '' : 'es' }}.
          <template v-if="validation.normalizedGlyphCount">
            Repaired {{ validation.normalizedGlyphCount.toLocaleString() }} outline{{ validation.normalizedGlyphCount === 1 ? '' : 's' }} for interpolation.
          </template>
          <template v-if="validation.frozenGlyphCount">
            Kept {{ validation.frozenGlyphCount.toLocaleString() }} glyph{{ validation.frozenGlyphCount === 1 ? '' : 's' }} at the default outline because they could not vary safely.
          </template>
        </span>
      </div>
      <section v-if="hasOutput && previewReady" class="variable-preview">
        <div class="variable-preview__heading">
          <div>
            <p class="variable-panel__eyebrow">
              Generated font
            </p>
            <h3>Preview</h3>
          </div>
          <span>Use the sliders to test the generated font.</span>
        </div>
        <label class="variable-preview__text">
          <span>Sample text</span>
          <input v-model="previewText" type="text">
        </label>
        <p class="variable-preview__sample" :style="previewStyle">
          {{ previewText || ' ' }}
        </p>
        <div class="variable-preview__sliders">
          <label v-for="(axis, index) in axes" :key="axis.tag">
            <span><strong>{{ axis.name || axis.tag }}</strong><output>{{ previewLocation[index] ?? axis.default }}</output></span>
            <input
              v-model.number="previewLocation[index]"
              type="range"
              :min="axis.minimum"
              :max="axis.maximum"
              :step="previewStep(axis)"
            >
          </label>
        </div>
      </section>
      <div v-else-if="hasOutput && previewError" class="variable-result variable-result--error">
        <UIcon name="i-lucide-triangle-alert" />
        <span>{{ previewError }}</span>
      </div>
      <div v-if="status === 'error'" class="variable-result variable-result--error">
        <UIcon name="i-lucide-triangle-alert" />
        <span>{{ error }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.variable-font-converter { font-family: inherit; }
.variable-dropzone { display: grid; place-items: center; gap: .45rem; min-height: 13rem; padding: 2rem; text-align: center; cursor: pointer; border: 1.5px dashed var(--ui-border); border-radius: 1rem; background: color-mix(in srgb, var(--ui-bg-elevated) 60%, transparent); transition: border-color .15s, background .15s, transform .15s; }
.variable-dropzone:hover, .variable-dropzone--active { border-color: var(--ui-primary); background: color-mix(in srgb, var(--ui-primary) 8%, var(--ui-bg-elevated)); transform: translateY(-1px); }
.variable-dropzone__icon { width: 2.5rem; height: 2.5rem; color: var(--ui-primary); }
.variable-dropzone__title { margin: 0; font-size: 1.05rem; font-weight: 650; color: var(--ui-text-highlighted); }
.variable-dropzone__subtitle { margin: 0; font-size: .9rem; color: var(--ui-text-muted); }
.variable-workspace { display: grid; gap: 1rem; margin-top: 1rem; }
.variable-panel { padding: 1rem; border: 1px solid var(--ui-border); border-radius: .9rem; background: var(--ui-bg-elevated); }
.variable-panel__heading { display: flex; align-items: center; justify-content: space-between; gap: 1rem; margin-bottom: 1rem; }
.variable-panel__heading h3 { margin: 0; font-size: 1rem; font-weight: 650; color: var(--ui-text-highlighted); }
.variable-panel__buttons { display: flex; flex-wrap: wrap; gap: .5rem; justify-content: flex-end; }
.variable-panel__eyebrow { margin: 0 0 .15rem; color: var(--ui-text-muted); font-size: .72rem; font-weight: 700; letter-spacing: .08em; text-transform: uppercase; }
.variable-panel__count { color: var(--ui-text-muted); font-size: .85rem; }
.variable-detection { margin: -.25rem 0 0; color: var(--ui-text-muted); font-size: .82rem; }
.axis-list, .master-list { display: grid; gap: .65rem; }
.axis-row { display: grid; grid-template-columns: .72fr 1.35fr repeat(3, .82fr) auto; gap: .55rem; align-items: end; }
.axis-row label, .master-row label, .repair-row label, .variable-output-name { display: grid; gap: .28rem; min-width: 0; }
.axis-row label span, .master-row label span, .repair-row label span, .variable-output-name span { color: var(--ui-text-muted); font-size: .72rem; font-weight: 600; }
.axis-row input, .axis-row select, .master-row input, .repair-row input, .repair-row select, .variable-output-name input { width: 100%; min-width: 0; padding: .48rem .6rem; color: var(--ui-text-highlighted); border: 1px solid var(--ui-border); border-radius: .45rem; background: var(--ui-bg); outline: none; }
.axis-row input:focus, .axis-row select:focus, .master-row input:focus, .repair-row input:focus, .repair-row select:focus, .variable-output-name input:focus { border-color: var(--ui-primary); box-shadow: 0 0 0 2px color-mix(in srgb, var(--ui-primary) 18%, transparent); }
.master-row { display: grid; grid-template-columns: minmax(11rem, 1.4fr) minmax(8rem, .9fr) minmax(11rem, 1.3fr) auto; gap: .75rem; align-items: end; padding: .75rem; border-radius: .65rem; background: color-mix(in srgb, var(--ui-bg) 88%, transparent); }
.master-row__file { display: flex; gap: .55rem; align-items: center; min-width: 0; padding-bottom: .25rem; }
.master-row__file > :first-child { flex: none; color: var(--ui-primary); width: 1.25rem; height: 1.25rem; }
.master-row__file div { display: grid; min-width: 0; }
.master-row__file strong { overflow: hidden; color: var(--ui-text-highlighted); font-size: .86rem; text-overflow: ellipsis; white-space: nowrap; }
.master-row__file span { color: var(--ui-text-muted); font-size: .75rem; }
.master-row__locations { display: grid; grid-template-columns: repeat(auto-fit, minmax(4.7rem, 1fr)); gap: .45rem; }
.master-row__locations label { display: grid; gap: .28rem; }
.master-row__locations span { color: var(--ui-text-muted); font-size: .72rem; font-weight: 600; }
.repair-list { display: grid; gap: .65rem; margin-top: .8rem; }
.repair-row { display: grid; grid-template-columns: minmax(7rem, .9fr) minmax(9rem, 1fr) repeat(4, minmax(6rem, .8fr)) auto; gap: .55rem; align-items: end; padding: .75rem; border-radius: .65rem; background: color-mix(in srgb, var(--ui-bg) 88%, transparent); }
.variable-actions { display: flex; align-items: end; justify-content: space-between; gap: 1rem; padding: 1rem; border: 1px solid var(--ui-border); border-radius: .9rem; background: color-mix(in srgb, var(--ui-primary) 5%, var(--ui-bg-elevated)); }
.variable-actions > div:first-child { display: grid; gap: .45rem; max-width: 32rem; }
.variable-actions p { margin: 0; color: var(--ui-text-muted); font-size: .82rem; line-height: 1.4; }
.variable-output-name { max-width: 22rem; }
.variable-actions__buttons { display: flex; flex-wrap: wrap; gap: .55rem; justify-content: flex-end; }
.variable-result { display: flex; gap: .55rem; align-items: flex-start; padding: .8rem 1rem; border-radius: .75rem; font-size: .9rem; }
.variable-result > :first-child { width: 1.15rem; height: 1.15rem; flex: none; margin-top: .05rem; }
.variable-result--success { color: var(--ui-success); background: color-mix(in srgb, var(--ui-success) 12%, transparent); }
.variable-result--error { color: var(--ui-error); background: color-mix(in srgb, var(--ui-error) 10%, transparent); }
.variable-preview { display: grid; gap: 1rem; padding: 1rem; border: 1px solid var(--ui-border); border-radius: .9rem; background: var(--ui-bg-elevated); }
.variable-preview__heading { display: flex; align-items: end; justify-content: space-between; gap: 1rem; }
.variable-preview__heading h3 { margin: 0; font-size: 1rem; font-weight: 650; color: var(--ui-text-highlighted); }
.variable-preview__heading > span { color: var(--ui-text-muted); font-size: .82rem; }
.variable-preview__text { display: grid; gap: .28rem; }
.variable-preview__text > span, .variable-preview__sliders label > span { color: var(--ui-text-muted); font-size: .72rem; font-weight: 600; }
.variable-preview__text input { width: 100%; padding: .48rem .6rem; color: var(--ui-text-highlighted); border: 1px solid var(--ui-border); border-radius: .45rem; background: var(--ui-bg); outline: none; }
.variable-preview__text input:focus { border-color: var(--ui-primary); box-shadow: 0 0 0 2px color-mix(in srgb, var(--ui-primary) 18%, transparent); }
.variable-preview__sample { min-height: 8rem; margin: 0; padding: 1rem; overflow-wrap: anywhere; color: var(--ui-text-highlighted); border-radius: .65rem; background: var(--ui-bg); font-size: clamp(1.8rem, 4vw, 3.8rem); line-height: 1.08; }
.variable-preview__sliders { display: grid; grid-template-columns: repeat(auto-fit, minmax(13rem, 1fr)); gap: .8rem; }
.variable-preview__sliders label { display: grid; gap: .45rem; }
.variable-preview__sliders label > span { display: flex; justify-content: space-between; gap: 1rem; }
.variable-preview__sliders strong { color: var(--ui-text-highlighted); font-weight: 650; }
.variable-preview__sliders output { color: var(--ui-primary); font-variant-numeric: tabular-nums; }
.variable-preview__sliders input { width: 100%; accent-color: var(--ui-primary); }
@media (max-width: 800px) { .axis-row { grid-template-columns: repeat(2, minmax(0, 1fr)); } .axis-row > :last-child { justify-self: end; } .master-row, .repair-row { grid-template-columns: 1fr; } .variable-panel__buttons { justify-content: flex-start; } .variable-actions { align-items: stretch; flex-direction: column; } .variable-actions__buttons { justify-content: flex-start; } }
</style>
