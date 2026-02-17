<script setup lang="ts">
/**
 * JoinDialog - Dialog for joining a chat with profile selection
 *
 * Uses registry primitives for MtDialog, MtRadioButton, MtCheckbox, MtButton
 */

import { ref, computed } from 'vue'
import type { JoinDialogRequest } from '../../types'
import { useI18n } from '../../i18n'
import { useRegistry } from '../../registry'
import Icon from '../Icon.vue'

const { t, tt } = useI18n()
const { MtDialog, MtRadioButton, MtCheckbox, MtButton } = useRegistry()

const props = defineProps<{
  show: boolean
  profileName: string
  company: string
  email?: string
  phone?: string
  loading?: boolean
  theme?: string
}>()

const emit = defineEmits<{
  cancel: []
  join: [data: JoinDialogRequest]
}>()

const nameMode = ref<'profile' | 'anonymous'>('profile')
const showEmail = ref(true)
const showPhone = ref(true)

const anonymousName = computed(() => tt('user.anonymous', { company: props.company }))

const selectedName = computed(() =>
  nameMode.value === 'profile' ? props.profileName : anonymousName.value
)

function handleJoin() {
  const joinData: JoinDialogRequest = {
    display_name: selectedName.value,
    company: props.company,
    email: showEmail.value ? props.email : undefined,
    phone: showPhone.value ? props.phone : undefined,
  }
  emit('join', joinData)
}

function handleClose() {
  emit('cancel')
}
</script>

<template>
  <component
    :is="MtDialog"
    :visible="show"
    :header="t.joinDialog.title"
    max-width="380px"
    :theme="theme || 'light'"
    @update:visible="!$event && handleClose()"
    @close="handleClose"
  >
    <div class="join-dialog__body">
      <!-- Name selection -->
      <div class="join-dialog__field">
        <label class="join-dialog__label">{{ t.joinDialog.displayName }}</label>
        <div class="join-dialog__radio-group">
          <component
            :is="MtRadioButton"
            v-model="nameMode"
            value="profile"
            :label="profileName"
            name="nameMode"
          />
          <component
            :is="MtRadioButton"
            v-model="nameMode"
            value="anonymous"
            :label="anonymousName"
            name="nameMode"
          />
        </div>
      </div>

      <!-- Company (read-only) -->
      <div class="join-dialog__field">
        <label class="join-dialog__label">{{ t.joinDialog.company }}</label>
        <input
          type="text"
          class="join-dialog__input join-dialog__input--disabled"
          :value="company"
          disabled
        />
      </div>

      <!-- Contact info toggles -->
      <div v-if="email || phone" class="join-dialog__field">
        <label class="join-dialog__label">{{ t.joinDialog.showContacts }}</label>
        <div class="join-dialog__toggles">
          <label v-if="email" class="join-dialog__toggle-row">
            <component
              :is="MtCheckbox"
              v-model="showEmail"
            />
            <span class="join-dialog__toggle-label">
              <Icon name="email" :size="14" />
              {{ email }}
            </span>
          </label>
          <label v-if="phone" class="join-dialog__toggle-row">
            <component
              :is="MtCheckbox"
              v-model="showPhone"
            />
            <span class="join-dialog__toggle-label">
              <Icon name="phone" :size="14" />
              {{ phone }}
            </span>
          </label>
        </div>
      </div>
    </div>

    <template #footer>
      <component
        :is="MtButton"
        variant="secondary"
        @click="handleClose"
      >
        {{ t.buttons.cancel }}
      </component>
      <component
        :is="MtButton"
        variant="primary"
        :disabled="loading"
        :loading="loading"
        @click="handleJoin"
      >
        {{ t.buttons.join }}
      </component>
    </template>
  </component>
</template>

<style scoped>
.join-dialog__body {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.join-dialog__field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.join-dialog__label {
  font-size: 11px;
  font-weight: 600;
  color: var(--mtchat-text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.join-dialog__input {
  font-size: 14px;
  padding: 8px 10px;
  border: 1px solid var(--mtchat-input-border);
  border-radius: 6px;
  background: var(--mtchat-input-bg);
  color: var(--mtchat-text);
  font-family: inherit;
}

.join-dialog__input--disabled {
  background: var(--mtchat-hover);
  color: var(--mtchat-text-secondary);
  cursor: not-allowed;
}

.join-dialog__radio-group {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.join-dialog__toggles {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.join-dialog__toggle-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 0;
}

.join-dialog__toggle-label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  color: var(--mtchat-text);
}

.join-dialog__toggle-label svg {
  color: var(--mtchat-text-secondary);
  flex-shrink: 0;
}
</style>
