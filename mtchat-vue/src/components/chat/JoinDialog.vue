<template>
  <Teleport to="body">
    <Transition name="join-dialog">
      <div v-if="show" :class="['join-dialog-overlay', `join-dialog--${theme || 'light'}`]" @click.self="$emit('cancel')">
        <div class="join-dialog">
          <div class="join-dialog__header">
            <h2 class="join-dialog__title">Присоединиться к чату</h2>
            <button
              class="join-dialog__close"
              @click="$emit('cancel')"
              title="Отмена"
            >
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <line x1="18" y1="6" x2="6" y2="18"/>
                <line x1="6" y1="6" x2="18" y2="18"/>
              </svg>
            </button>
          </div>

          <div class="join-dialog__body">
            <!-- Name selection -->
            <div class="join-dialog__field">
              <label class="join-dialog__label">Отображаемое имя</label>
              <div class="join-dialog__radio-group">
                <label class="join-dialog__radio">
                  <input
                    type="radio"
                    v-model="nameMode"
                    value="profile"
                    name="nameMode"
                  />
                  <span class="join-dialog__radio-label">{{ profileName }}</span>
                </label>
                <label class="join-dialog__radio">
                  <input
                    type="radio"
                    v-model="nameMode"
                    value="anonymous"
                    name="nameMode"
                  />
                  <span class="join-dialog__radio-label">{{ anonymousName }}</span>
                </label>
              </div>
            </div>

            <!-- Company (read-only) -->
            <div class="join-dialog__field">
              <label class="join-dialog__label">Компания</label>
              <input
                type="text"
                class="join-dialog__input join-dialog__input--disabled"
                :value="company"
                disabled
              />
            </div>

            <!-- Contact info toggles -->
            <div v-if="email || phone" class="join-dialog__field">
              <label class="join-dialog__label">Показать контакты</label>
              <div class="join-dialog__toggles">
                <label v-if="email" class="join-dialog__toggle">
                  <input type="checkbox" v-model="showEmail" />
                  <span class="join-dialog__toggle-label">
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z"/>
                      <polyline points="22,6 12,13 2,6"/>
                    </svg>
                    {{ email }}
                  </span>
                </label>
                <label v-if="phone" class="join-dialog__toggle">
                  <input type="checkbox" v-model="showPhone" />
                  <span class="join-dialog__toggle-label">
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M22 16.92v3a2 2 0 01-2.18 2 19.79 19.79 0 01-8.63-3.07 19.5 19.5 0 01-6-6 19.79 19.79 0 01-3.07-8.67A2 2 0 014.11 2h3a2 2 0 012 1.72 12.84 12.84 0 00.7 2.81 2 2 0 01-.45 2.11L8.09 9.91a16 16 0 006 6l1.27-1.27a2 2 0 012.11-.45 12.84 12.84 0 002.81.7A2 2 0 0122 16.92z"/>
                    </svg>
                    {{ phone }}
                  </span>
                </label>
              </div>
            </div>
          </div>

          <div class="join-dialog__footer">
            <button
              class="join-dialog__btn join-dialog__btn--secondary"
              @click="$emit('cancel')"
            >
              Отмена
            </button>
            <button
              class="join-dialog__btn join-dialog__btn--primary"
              @click="handleJoin"
              :disabled="loading"
            >
              {{ loading ? 'Присоединение...' : 'Присоединиться' }}
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, computed, watch, onUnmounted } from 'vue'
import type { JoinDialogRequest } from '../../types'

const props = defineProps<{
  show: boolean
  profileName: string
  company: string
  email?: string
  phone?: string
  loading?: boolean
  theme?: 'light' | 'dark'
}>()

const emit = defineEmits<{
  (e: 'cancel'): void
  (e: 'join', data: JoinDialogRequest): void
}>()

const nameMode = ref<'profile' | 'anonymous'>('profile')
const showEmail = ref(true)
const showPhone = ref(true)

const anonymousName = computed(() => `Сотрудник компании ${props.company}`)

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

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape' && props.show) {
    emit('cancel')
  }
}

watch(() => props.show, (show) => {
  if (show) {
    document.addEventListener('keydown', handleKeydown)
  } else {
    document.removeEventListener('keydown', handleKeydown)
  }
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
})
</script>

<style scoped>
.join-dialog-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

/* Light theme */
.join-dialog--light {
  --mtchat-bg: #ffffff;
  --mtchat-text: #334155;
  --mtchat-text-secondary: #64748b;
  --mtchat-border: #e2e8f0;
  --mtchat-hover: #f1f5f9;
  --mtchat-primary: #3B82F6;
  --mtchat-input-bg: #ffffff;
  --mtchat-input-border: #d1d5db;
}

/* Dark theme */
.join-dialog--dark {
  --mtchat-bg: #1f2937;
  --mtchat-text: #f8fafc;
  --mtchat-text-secondary: #94a3b8;
  --mtchat-border: #374151;
  --mtchat-hover: #374151;
  --mtchat-primary: #60a5fa;
  --mtchat-input-bg: #111827;
  --mtchat-input-border: #374151;
}

.join-dialog {
  background: var(--mtchat-bg);
  border-radius: 12px;
  width: 100%;
  max-width: 380px;
  margin: 16px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
  color: var(--mtchat-text);
}

.join-dialog__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 16px;
  border-bottom: 1px solid var(--mtchat-border);
}

.join-dialog__title {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
}

.join-dialog__close {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: none;
  border-radius: 6px;
  cursor: pointer;
  color: var(--mtchat-text-secondary);
  transition: background-color 0.2s, color 0.2s;
}

.join-dialog__close:hover {
  background: var(--mtchat-hover);
  color: var(--mtchat-text);
}

.join-dialog__body {
  padding: 16px;
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

.join-dialog__radio {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 0;
  cursor: pointer;
}

.join-dialog__radio:hover .join-dialog__radio-label {
  color: var(--mtchat-text);
}

.join-dialog__radio input[type="radio"] {
  width: 16px;
  height: 16px;
  margin: 0;
  accent-color: var(--mtchat-primary);
  cursor: pointer;
}

.join-dialog__radio-label {
  font-size: 14px;
  color: var(--mtchat-text);
}

.join-dialog__toggles {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.join-dialog__toggle {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 0;
  cursor: pointer;
}

.join-dialog__toggle:hover .join-dialog__toggle-label {
  color: var(--mtchat-text);
}

.join-dialog__toggle input[type="checkbox"] {
  width: 16px;
  height: 16px;
  margin: 0;
  accent-color: var(--mtchat-primary);
  cursor: pointer;
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

.join-dialog__footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 12px 16px;
  border-top: 1px solid var(--mtchat-border);
}

.join-dialog__btn {
  padding: 8px 16px;
  border-radius: 6px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: background-color 0.2s, opacity 0.2s;
  border: none;
}

.join-dialog__btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.join-dialog__btn--secondary {
  background: var(--mtchat-hover);
  color: var(--mtchat-text);
}

.join-dialog__btn--secondary:hover:not(:disabled) {
  background: var(--mtchat-border);
}

.join-dialog__btn--primary {
  background: var(--mtchat-primary);
  color: white;
}

.join-dialog__btn--primary:hover:not(:disabled) {
  opacity: 0.9;
}

/* Transition */
.join-dialog-enter-active,
.join-dialog-leave-active {
  transition: opacity 0.2s ease;
}

.join-dialog-enter-active .join-dialog,
.join-dialog-leave-active .join-dialog {
  transition: transform 0.2s ease, opacity 0.2s ease;
}

.join-dialog-enter-from,
.join-dialog-leave-to {
  opacity: 0;
}

.join-dialog-enter-from .join-dialog,
.join-dialog-leave-to .join-dialog {
  transform: scale(0.95);
  opacity: 0;
}
</style>
