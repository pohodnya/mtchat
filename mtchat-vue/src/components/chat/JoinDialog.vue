<template>
  <Teleport to="body">
    <Transition name="join-dialog">
      <div v-if="show" class="join-dialog-overlay" @click.self="$emit('cancel')">
        <div class="join-dialog">
          <div class="join-dialog__header">
            <h2 class="join-dialog__title">Присоединиться к чату</h2>
            <button
              class="join-dialog__close"
              @click="$emit('cancel')"
              title="Отмена"
            >
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
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
              <div class="join-dialog__value">{{ company }}</div>
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
import { ref, computed } from 'vue'
import type { JoinDialogRequest } from '../../types'

const props = defineProps<{
  show: boolean
  profileName: string
  company: string
  email?: string
  phone?: string
  loading?: boolean
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

.join-dialog {
  background: var(--mtchat-bg, #ffffff);
  border-radius: 12px;
  width: 100%;
  max-width: 420px;
  margin: 16px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
  color: var(--mtchat-text, #1a1a1a);
}

.join-dialog__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--mtchat-border, #e5e5e5);
}

.join-dialog__title {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
}

.join-dialog__close {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: none;
  border-radius: 6px;
  cursor: pointer;
  color: var(--mtchat-text-secondary, #666);
  transition: background-color 0.2s, color 0.2s;
}

.join-dialog__close:hover {
  background: var(--mtchat-hover, #f0f0f0);
  color: var(--mtchat-text, #1a1a1a);
}

.join-dialog__body {
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.join-dialog__field {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.join-dialog__label {
  font-size: 13px;
  font-weight: 600;
  color: var(--mtchat-text-secondary, #666);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.join-dialog__value {
  font-size: 15px;
  color: var(--mtchat-text, #1a1a1a);
  padding: 10px 12px;
  background: var(--mtchat-hover, #f0f0f0);
  border-radius: 8px;
}

.join-dialog__radio-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.join-dialog__radio {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  background: var(--mtchat-hover, #f0f0f0);
  border-radius: 8px;
  cursor: pointer;
  transition: background-color 0.2s;
}

.join-dialog__radio:hover {
  background: var(--mtchat-border, #e5e5e5);
}

.join-dialog__radio input[type="radio"] {
  width: 18px;
  height: 18px;
  margin: 0;
  accent-color: var(--mtchat-primary, #007AFF);
}

.join-dialog__radio-label {
  font-size: 15px;
  color: var(--mtchat-text, #1a1a1a);
}

.join-dialog__toggles {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.join-dialog__toggle {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  background: var(--mtchat-hover, #f0f0f0);
  border-radius: 8px;
  cursor: pointer;
  transition: background-color 0.2s;
}

.join-dialog__toggle:hover {
  background: var(--mtchat-border, #e5e5e5);
}

.join-dialog__toggle input[type="checkbox"] {
  width: 18px;
  height: 18px;
  margin: 0;
  accent-color: var(--mtchat-primary, #007AFF);
}

.join-dialog__toggle-label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 14px;
  color: var(--mtchat-text, #1a1a1a);
}

.join-dialog__toggle-label svg {
  color: var(--mtchat-text-secondary, #666);
}

.join-dialog__footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  padding: 16px 20px;
  border-top: 1px solid var(--mtchat-border, #e5e5e5);
}

.join-dialog__btn {
  padding: 10px 20px;
  border-radius: 8px;
  font-size: 14px;
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
  background: var(--mtchat-hover, #f0f0f0);
  color: var(--mtchat-text, #1a1a1a);
}

.join-dialog__btn--secondary:hover:not(:disabled) {
  background: var(--mtchat-border, #e5e5e5);
}

.join-dialog__btn--primary {
  background: var(--mtchat-primary, #007AFF);
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
