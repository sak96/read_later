<script setup lang="ts">
import { ref, onMounted, inject } from 'vue'
import { invokeParse, invokeParseLogError, invokeNoParseLogError } from '../composables/useTauri'
import { Pencil, Plus, Trash2 } from 'lucide-vue-next'
import type { AlertContext } from '../types'
import Fab from '../layouts/Fab.vue'
import SettingsButton from '../components/SettingsButton.vue'
import HomeButton from '../components/HomeButton.vue'
import ConfirmModal from '../components/ConfirmModal.vue'

interface PronunciationRule {
  match_pattern: string
  replacement: string
  is_regex: boolean
}

const alertContext = inject<AlertContext | null>('alert')
const rules = ref<PronunciationRule[]>([])

const editingRule = ref<PronunciationRule>({ match_pattern: '', replacement: '', is_regex: false })
const isNewRule = ref(false)
const showEditor = ref(false)
const showDeleteConfirm = ref(false)
const deletingPattern = ref('')

async function loadRules() {
  rules.value = await invokeParseLogError<PronunciationRule[]>('get_pronunciation_rules') || []
}

function openEditor(rule?: PronunciationRule) {
  if (rule) {
    editingRule.value = { ...rule }
    isNewRule.value = false
  }
  else {
    editingRule.value = { match_pattern: '', replacement: '', is_regex: false }
    isNewRule.value = true
  }
  showEditor.value = true
}

function closeEditor() {
  showEditor.value = false
}

async function saveRule() {
  if (!editingRule.value.match_pattern) return
  try {
    await invokeParse('save_pronunciation_rule', {
      matchPattern: editingRule.value.match_pattern,
      replacement: editingRule.value.replacement,
      isRegex: editingRule.value.is_regex,
    })
  }
  catch (err) {
    alertContext?.updateAlertContext?.('error', `${err}`)
    return
  }
  showEditor.value = false
  await loadRules()
}

function confirmDelete(matchPattern: string) {
  deletingPattern.value = matchPattern
  showDeleteConfirm.value = true
}

async function deleteRule() {
  await invokeNoParseLogError('delete_pronunciation_rule', { matchPattern: deletingPattern.value })
  showDeleteConfirm.value = false
  await loadRules()
}

onMounted(loadRules)
</script>

<template>
  <main class="container page">
    <article>
      <h4>
        <span data-i18n="pronunciation_rules" />
      </h4>
      <div role="group">
        <button
          type="button"
          class="outline"
          @click="openEditor()"
        >
          <Plus />
          <span data-i18n="add_new_rule" />
        </button>
      </div>

      <div>
        <article
          v-for="rule in rules"
          :key="rule.match_pattern"
        >
          <header>
            {{ rule.match_pattern }}
            <sub v-if="rule.is_regex">&nbsp;regex</sub>
          </header>
          <p>{{ rule.replacement }}</p>
          <footer>
            <div role="group">
              <button
                type="button"
                @click="openEditor(rule)"
              >
                <Pencil />
              </button>
              <button
                type="button"
                class="secondary"
                @click="confirmDelete(rule.match_pattern)"
              >
                <Trash2 />
              </button>
            </div>
          </footer>
        </article>
      </div>

      <Fab>
        <SettingsButton />
        <HomeButton />
      </Fab>
    </article>

    <ConfirmModal
      i18n-key="pronunciation_rules"
      message=""
      :show="showEditor"
      @confirm="saveRule"
      @close="closeEditor"
    >
      <label>
        <span data-i18n="match_pattern" />
        <input
          v-model="editingRule.match_pattern"
          type="text"
          :disabled="!isNewRule"
        >
      </label>
      <label>
        <span data-i18n="replacement" />
        <input
          v-model="editingRule.replacement"
          type="text"
        >
      </label>
      <label>
        <input
          v-model="editingRule.is_regex"
          type="checkbox"
          role="switch"
        >
        <span data-i18n="regex" />
      </label>
    </ConfirmModal>

    <ConfirmModal
      :icon="Trash2"
      i18n-key="delete_rule"
      :message="deletingPattern"
      :show="showDeleteConfirm"
      @confirm="deleteRule"
      @close="showDeleteConfirm = false"
    />
  </main>
</template>
